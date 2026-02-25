use crate::OmniDriveServer;
use crate::config::AppConfig;
use crate::sandbox::{
    validate_path, validate_writable, validate_destructive,
    is_supported_extension, is_binary_file, is_pdf,
};
use rmcp::{tool, model::CallToolResult, model::Content, ErrorData};
use rmcp::handler::server::wrapper::Parameters;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, BufRead, Write};
use std::sync::Arc;


use tokio::sync::RwLock;
use base64::{Engine as _, engine::general_purpose};

fn success_log(
    tool: &str,
    category: &str,
    path: Option<&str>,
    summary: &str,
    contents: Vec<Content>,
) -> CallToolResult {
    crate::activity::log_activity(tool, category, path, summary);
    CallToolResult::success(contents)
}

// ─── Tool Parameters ───

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct ListDirectoryParams {
    path: String,
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_page_size")]
    page_size: usize,
    /// If true, list recursively with tree structure
    #[serde(default)]
    recursive: bool,
    /// Max depth when recursive=true (default 3)
    #[serde(default = "default_max_depth")]
    max_depth: usize,
}

fn default_page() -> usize { 1 }
fn default_page_size() -> usize { 50 }
fn default_max_depth() -> usize { 3 }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct ReadFileParams {
    path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct WriteFileParams {
    path: String,
    content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct SearchFilesParams {
    pattern: String,
    root_path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct GrepContentParams {
    /// String or regex pattern to search for inside files
    pattern: String,
    /// Root directory to search within
    root_path: String,
    /// Treat pattern as regex (default: false, literal string match)
    #[serde(default)]
    is_regex: bool,
    /// Case-insensitive matching (default: false)
    #[serde(default)]
    case_insensitive: bool,
    /// Max results to return (default: 50)
    #[serde(default = "default_max_results")]
    max_results: usize,
    /// Only search files with these extensions (e.g. ["rs", "py"])
    #[serde(default)]
    include_extensions: Option<Vec<String>>,
}

fn default_max_results() -> usize { 50 }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct ReadLinesParams {
    /// Path to the file
    path: String,
    /// Start line (1-indexed, inclusive). If omitted with no tail, reads from line 1.
    start_line: Option<usize>,
    /// End line (1-indexed, inclusive). If omitted, reads to end or 100 lines from start.
    end_line: Option<usize>,
    /// Read the last N lines of the file (overrides start_line/end_line)
    tail: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct MoveFileParams {
    source: String,
    destination: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct DeleteFileParams {
    path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct CopyFileParams {
    source: String,
    destination: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct GetFileInfoParams {
    path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct BatchReadParams {
    /// List of file paths to read
    paths: Vec<String>,
    /// Max total content size in MB (default: 5)
    #[serde(default = "default_batch_max_size")]
    max_total_size_mb: f64,
}

fn default_batch_max_size() -> f64 { 5.0 }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct ZipFilesParams {
    /// Files and/or directories to include in the archive
    paths: Vec<String>,
    /// Output zip file path
    output_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct UnzipFilesParams {
    /// Path to the zip archive
    archive_path: String,
    /// Directory to extract into
    destination: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct PatchOp {
    /// The text to search for in the file
    search: String,
    /// The replacement text
    replace: String,
    /// Treat search as regex (default: false)
    #[serde(default)]
    regex: bool,
    /// Max number of replacements (omit or 0 for all occurrences)
    #[serde(default)]
    count: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct LinePatchOp {
    /// Start line (1-indexed, inclusive)
    start_line: usize,
    /// End line (1-indexed, inclusive)
    end_line: usize,
    /// Replacement content for the specified line range
    content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct PatchFileParams {
    path: String,
    /// Search-and-replace operations (applied sequentially)
    #[serde(default)]
    search_replace: Vec<PatchOp>,
    /// Line-range replacement operations (applied after search_replace)
    #[serde(default)]
    line_replace: Vec<LinePatchOp>,
}

// ─── Tool Implementations ───

#[rmcp::tool_router]
impl OmniDriveServer {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            tool_router: Self::tool_router(),
        }
    }

    // ────────────────────────────────────────────────────────
    // 1. list_directory (enhanced with recursive option)
    // ────────────────────────────────────────────────────────

    #[tool(description = "List files in a directory. Returns names, types, and sizes. Paginated. Set recursive=true with max_depth to get a tree structure.")]
    async fn list_directory(&self, params: Parameters<ListDirectoryParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let validated = validate_path(&args.path, &config).map_err(|e| ErrorData::internal_error(e, None))?;
        let dir_path = validated.canonical_path;

        if !dir_path.is_dir() {
            return Err(ErrorData::internal_error(format!("Path is not a directory: {}", args.path), None));
        }

        if args.recursive {
            return self.list_directory_recursive(&dir_path, &args, &config).await;
        }

        let mut entries = Vec::new();
        match fs::read_dir(&dir_path) {
            Ok(dir_entries) => {
                for entry in dir_entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    let is_dir = path.is_dir();

                    if !is_dir && !is_supported_extension(&name) {
                        continue;
                    }

                    // Check .mcpignore for each entry
                    let path_str = path.to_string_lossy().to_string();
                    if validate_path(&path_str, &config).is_err() {
                        continue;
                    }

                    let size = if is_dir { 0 } else { entry.metadata().map(|m| m.len()).unwrap_or(0) };
                    entries.push((name, is_dir, size));
                }
            }
            Err(e) => return Err(ErrorData::internal_error(format!("Failed to read directory: {}", e), None)),
        }

        entries.sort_by(|a, b| {
            match (a.1, b.1) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
            }
        });

        let total_items = entries.len();
        let page_size = args.page_size.clamp(1, 100);
        let page = args.page.max(1);
        let start_idx = (page - 1) * page_size;

        if start_idx >= total_items && total_items > 0 {
             return

        Ok(success_log("list_directory", "read", Some(&args.path.clone()), "Listed directory items", vec![Content::text(format!(
                "Page {} is out of range. Total items: {} ({} pages)",
                page, total_items, (total_items + page_size - 1) / page_size
            ))]));
        }

        let paged_entries = entries.into_iter().skip(start_idx).take(page_size);

        let mut output = String::new();
        output.push_str(&format!("Directory listing for: {}\n", args.path));
        output.push_str(&format!("Page {} of {} ({} items)\n\n",
            page, (total_items + page_size - 1) / page_size, total_items));
        output.push_str("Type  | Size       | Name\n");
        output.push_str("------+------------+---------------------------------------------\n");

        for (name, is_dir, size) in paged_entries {
            let type_str = if is_dir { "<DIR>" } else { "FIL" };
            let size_str = if is_dir { "-".to_string() } else { format_size(size) };
            output.push_str(&format!("{:<5} | {:<10} | {}\n", type_str, size_str, name));
        }

        Ok(success_log("list_directory", "read", Some(&args.path.clone()), "Listed directory items", vec![Content::text(output)]))
    }

    // ────────────────────────────────────────────────────────
    // 2. read_file
    // ────────────────────────────────────────────────────────

    #[tool(description = "Read file content. Supports text, images (base64), and PDFs (text extract).")]
    async fn read_file(&self, params: Parameters<ReadFileParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let validated = validate_path(&args.path, &config).map_err(|e| ErrorData::internal_error(e, None))?;
        let file_path = validated.canonical_path;

        if !file_path.exists() || !file_path.is_file() {
            return Err(ErrorData::internal_error(format!("File not found: {}", args.path), None));
        }

        let metadata = fs::metadata(&file_path).map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        if size_mb > config.max_file_size_mb as f64 {
             return Err(ErrorData::internal_error(
                format!("File too large: {:.2} MB (limit: {} MB). Use read_lines tool for partial reads.", size_mb, config.max_file_size_mb),
                None,
            ));
        }

        let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if is_pdf(filename) {
             match pdf_extract::extract_text(&file_path) {
                Ok(text) =>

        Ok(success_log("read_file", "read", Some(&args.path.clone()), "Read file contents", vec![Content::text(text)])),
                Err(e) => Err(ErrorData::internal_error(format!("Failed to extract PDF text: {}", e), None)),
            }
        } else if is_binary_file(filename) {
             let mut file = fs::File::open(&file_path).map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
             let mut buffer = Vec::new();
             file.read_to_end(&mut buffer).map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

             let encoded = general_purpose::STANDARD.encode(&buffer);
             let mime_type = mime_guess::from_path(&file_path).first_or_text_plain();

        Ok(success_log("read_file", "read", Some(&args.path.clone()), "Read file contents", vec![Content::text(format!(
                 "[Image/Binary content evaluated as base64]\ndata:{};base64,{}",
                 mime_type, encoded
             ))]))
        } else if is_supported_extension(filename) {
             match fs::read_to_string(&file_path) {
                Ok(content) =>

        Ok(success_log("read_file", "read", Some(&args.path.clone()), "Read file contents", vec![Content::text(content)])),
                Err(_) => {
                    let content_lossy = fs::read_to_string(&file_path).unwrap_or_default();

        Ok(success_log("read_file", "read", Some(&args.path.clone()), "Read file contents", vec![Content::text(content_lossy)]))
                }
            }
        } else {
            Err(ErrorData::internal_error(format!("Unsupported file type: {}", filename), None))
        }
    }

    // ────────────────────────────────────────────────────────
    // 3. write_file
    // ────────────────────────────────────────────────────────

    #[tool(description = "Create or overwrite a file. Requires Read/Write permission.")]
    async fn write_file(&self, params: Parameters<WriteFileParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let validated = validate_writable(&args.path, &config).map_err(|e| ErrorData::internal_error(e, None))?;
        let file_path = validated.canonical_path;

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| ErrorData::internal_error(format!("Failed to create parent dirs: {}", e), None))?;
        }

        match fs::write(&file_path, args.content) {
            Ok(_) =>

        Ok(success_log("write_file", "write", Some(&args.path.clone()), &format!("Wrote file: {}", args.path), vec![Content::text(format!("Successfully wrote to {}", args.path))])),
            Err(e) => Err(ErrorData::internal_error(format!("Failed to write file: {}", e), None)),
        }
    }

    // ────────────────────────────────────────────────────────
    // 4. search_files
    // ────────────────────────────────────────────────────────

    #[tool(description = "Search for files by glob pattern across shared folders.")]
    async fn search_files(&self, params: Parameters<SearchFilesParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let mut results = Vec::new();
        let pattern_str = args.pattern.trim();

        for folder in &config.folders {
            if !folder.enabled { continue; }
            if let Some(ref root) = args.root_path {
                 if !folder.path.starts_with(root) && !root.starts_with(&folder.path) {
                     continue;
                 }
            }

            let glob_pattern = if pattern_str.contains('/') || pattern_str.contains('\\') {
                format!("{}/{}", folder.path, pattern_str)
            } else {
                format!("{}/**/{}", folder.path, pattern_str)
            };

            match glob::glob(&glob_pattern) {
                Ok(paths) => {
                    for entry in paths {
                        if let Ok(path) = entry {
                            let path_str = path.to_string_lossy().to_string();
                            if path.is_file() && validate_path(&path_str, &config).is_ok() {
                                results.push(path_str);
                            }
                        }
                    }
                },
                Err(e) => eprintln!("Glob error: {}", e),
            }
        }

        if results.len() > 100 {
            let total = results.len();
            results.truncate(100);
            results.push(format!("... and {} more results", total - 100));
        }

        if results.is_empty() {

        Ok(success_log("search_files", "read", Some(&args.pattern.clone()), "Searched files", vec![Content::text("No matching files found.")]))
        } else {

        Ok(success_log("search_files", "read", Some(&args.pattern.clone()), "Searched files", vec![Content::text(results.join("\n"))]))
        }
    }

    // ────────────────────────────────────────────────────────
    // 5. grep_content — search inside file contents
    // ────────────────────────────────────────────────────────

    #[tool(description = "Search for a string or regex pattern inside file contents. Returns matching file paths, line numbers, and line content.")]
    async fn grep_content(&self, params: Parameters<GrepContentParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let validated = validate_path(&args.root_path, &config).map_err(|e| ErrorData::internal_error(e, None))?;
        let root = validated.canonical_path;

        if !root.is_dir() {
            return Err(ErrorData::internal_error(
                format!("root_path must be a directory: {}", args.root_path), None,
            ));
        }

        let max_results = if args.max_results == 0 { 50 } else { args.max_results.min(200) };

        // Build the regex matcher
        let re = if args.is_regex {
            let pattern = if args.case_insensitive {
                format!("(?i){}", args.pattern)
            } else {
                args.pattern.clone()
            };
            regex::Regex::new(&pattern).map_err(|e| {
                ErrorData::internal_error(format!("Invalid regex '{}': {}", args.pattern, e), None)
            })?
        } else {
            let escaped = regex::escape(&args.pattern);
            let pattern = if args.case_insensitive {
                format!("(?i){}", escaped)
            } else {
                escaped
            };
            regex::Regex::new(&pattern).map_err(|e| {
                ErrorData::internal_error(format!("Internal regex error: {}", e), None)
            })?
        };

        let mut results = Vec::new();
        let walker = walkdir::WalkDir::new(&root)
            .max_depth(20)
            .into_iter()
            .filter_map(|e| e.ok());

        'outer: for entry in walker {
            let path = entry.path();
            if !path.is_file() { continue; }

            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !is_supported_extension(filename) || is_binary_file(filename) { continue; }

            // Extension filter
            if let Some(ref exts) = args.include_extensions {
                let file_ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !exts.iter().any(|e| e.eq_ignore_ascii_case(file_ext)) {
                    continue;
                }
            }

            // Validate path is within sandbox
            let path_str = path.to_string_lossy().to_string();
            if validate_path(&path_str, &config).is_err() { continue; }

            // Check file size — skip very large files
            if let Ok(meta) = fs::metadata(path) {
                if meta.len() > (config.max_file_size_mb as u64) * 1024 * 1024 {
                    continue;
                }
            }

            // Read and search
            if let Ok(file) = fs::File::open(path) {
                let reader = std::io::BufReader::new(file);
                for (line_num, line) in reader.lines().enumerate() {
                    if let Ok(line) = line {
                        if re.is_match(&line) {
                            results.push(format!(
                                "{}:{}:{}",
                                path_str,
                                line_num + 1,
                                line.chars().take(200).collect::<String>()
                            ));
                            if results.len() >= max_results {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        if results.is_empty() {

        Ok(success_log("grep_content", "read", Some(&args.root_path.clone()), &format!("Grepped for {}", args.pattern), vec![Content::text(
                format!("No matches found for '{}' in {}", args.pattern, args.root_path),
            )]))
        } else {
            let header = format!("Found {} match(es) for '{}':\n\n", results.len(), args.pattern);

        Ok(success_log("grep_content", "read", Some(&args.root_path.clone()), &format!("Grepped for {}", args.pattern), vec![Content::text(
                format!("{}{}", header, results.join("\n")),
            )]))
        }
    }

    // ────────────────────────────────────────────────────────
    // 6. read_lines — read head/tail/range of a file
    // ────────────────────────────────────────────────────────

    #[tool(description = "Read specific lines from a file. Use start_line/end_line for a range, tail=N for last N lines, or omit all for first 100 lines. Returns line-numbered content and total line count.")]
    async fn read_lines(&self, params: Parameters<ReadLinesParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let validated = validate_path(&args.path, &config).map_err(|e| ErrorData::internal_error(e, None))?;
        let file_path = validated.canonical_path;

        if !file_path.exists() || !file_path.is_file() {
            return Err(ErrorData::internal_error(format!("File not found: {}", args.path), None));
        }

        let content = fs::read_to_string(&file_path).map_err(|e| {
            ErrorData::internal_error(format!("Cannot read file (binary?): {}. Use read_file for binary content.", e), None)
        })?;

        let all_lines: Vec<&str> = content.lines().collect();
        let total_lines = all_lines.len();

        let (start, end) = if let Some(tail_n) = args.tail {
            let n = tail_n.min(total_lines);
            (total_lines.saturating_sub(n), total_lines)
        } else {
            let s = args.start_line.unwrap_or(1).max(1).min(total_lines + 1) - 1;
            let e = args.end_line.unwrap_or(s + 100).min(total_lines);
            (s, e)
        };

        let mut output = String::new();
        output.push_str(&format!("File: {} ({} total lines)\n", args.path, total_lines));
        output.push_str(&format!("Showing lines {}-{}:\n\n", start + 1, end));

        for (i, line) in all_lines[start..end].iter().enumerate() {
            output.push_str(&format!("{:>6} | {}\n", start + i + 1, line));
        }

        Ok(success_log("read_lines", "read", Some(&args.path.clone()), "Read file lines", vec![Content::text(output)]))
    }

    // ────────────────────────────────────────────────────────
    // 7. move_file — move or rename a file/directory
    // ────────────────────────────────────────────────────────

    #[tool(description = "Move or rename a file or directory. Both source and destination must be in writable shared folders.")]
    async fn move_file(&self, params: Parameters<MoveFileParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let src_validated = validate_destructive(&args.source, &config)
            .map_err(|e| ErrorData::internal_error(e, None))?;
        let dst_validated = validate_writable(&args.destination, &config)
            .map_err(|e| ErrorData::internal_error(e, None))?;

        let src = src_validated.canonical_path;
        let dst = dst_validated.canonical_path;

        if dst.exists() {
            return Err(ErrorData::internal_error(
                format!("Destination already exists: {}. Delete it first or choose a different name.", args.destination),
                None,
            ));
        }

        // Create parent directories for destination
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ErrorData::internal_error(format!("Failed to create destination directory: {}", e), None)
            })?;
        }

        // Try rename first (same filesystem), fall back to copy+delete
        match fs::rename(&src, &dst) {
            Ok(_) =>

        Ok(success_log("move_file", "delete", Some(&args.source.clone()), &format!("Moved to {}", args.destination), vec![Content::text(
                format!("Moved {} → {}", args.source, args.destination),
            )])),
            Err(_) => {
                // Cross-device move: copy then delete
                if src.is_file() {
                    fs::copy(&src, &dst).map_err(|e| {
                        ErrorData::internal_error(format!("Failed to copy during move: {}", e), None)
                    })?;
                    fs::remove_file(&src).map_err(|e| {
                        ErrorData::internal_error(format!("Copied but failed to remove source: {}", e), None)
                    })?;
                } else {
                    return Err(ErrorData::internal_error(
                        "Cross-device directory moves are not supported. Copy manually and delete the source.",
                        None,
                    ));
                }

        Ok(success_log("move_file", "delete", Some(&args.source.clone()), &format!("Moved to {}", args.destination), vec![Content::text(
                    format!("Moved {} → {} (cross-device)", args.source, args.destination),
                )]))
            }
        }
    }

    // ────────────────────────────────────────────────────────
    // 8. delete_file — delete a file or empty directory
    // ────────────────────────────────────────────────────────

    #[tool(description = "Delete a file or empty directory. Requires Read/Write permission. Non-empty directories cannot be deleted (safety measure).")]
    async fn delete_file(&self, params: Parameters<DeleteFileParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let validated = validate_destructive(&args.path, &config)
            .map_err(|e| ErrorData::internal_error(e, None))?;
        let target = validated.canonical_path;

        if target.is_file() {
            fs::remove_file(&target).map_err(|e| {
                ErrorData::internal_error(format!("Failed to delete file: {}", e), None)
            })?;

        Ok(success_log("delete_file", "delete", Some(&args.path.clone()), "Deleted file/dir", vec![Content::text(format!("Deleted file: {}", args.path))]))
        } else if target.is_dir() {
            fs::remove_dir(&target).map_err(|e| {
                ErrorData::internal_error(
                    format!("Failed to delete directory: {}. Only empty directories can be deleted.", e),
                    None,
                )
            })?;

        Ok(success_log("delete_file", "delete", Some(&args.path.clone()), "Deleted file/dir", vec![Content::text(format!("Deleted empty directory: {}", args.path))]))
        } else {
            Err(ErrorData::internal_error(format!("Unknown file type at: {}", args.path), None))
        }
    }

    // ────────────────────────────────────────────────────────
    // 9. copy_file — copy a file
    // ────────────────────────────────────────────────────────

    #[tool(description = "Copy a file. Source must be readable, destination must be in a writable shared folder.")]
    async fn copy_file(&self, params: Parameters<CopyFileParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let src_validated = validate_path(&args.source, &config)
            .map_err(|e| ErrorData::internal_error(e, None))?;
        let dst_validated = validate_writable(&args.destination, &config)
            .map_err(|e| ErrorData::internal_error(e, None))?;

        let src = src_validated.canonical_path;
        let dst = dst_validated.canonical_path;

        if !src.is_file() {
            return Err(ErrorData::internal_error(
                format!("Source is not a file: {}. Only files can be copied.", args.source),
                None,
            ));
        }

        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ErrorData::internal_error(format!("Failed to create destination directory: {}", e), None)
            })?;
        }

        let bytes_copied = fs::copy(&src, &dst).map_err(|e| {
            ErrorData::internal_error(format!("Failed to copy file: {}", e), None)
        })?;

        Ok(success_log("copy_file", "write", Some(&args.destination.clone()), &format!("Copied from {}", args.source), vec![Content::text(
            format!("Copied {} → {} ({})", args.source, args.destination, format_size(bytes_copied)),
        )]))
    }

    // ────────────────────────────────────────────────────────
    // 10. get_file_info — metadata without reading content
    // ────────────────────────────────────────────────────────

    #[tool(description = "Get file or directory metadata (size, modified date, type, MIME) without reading content.")]
    async fn get_file_info(&self, params: Parameters<GetFileInfoParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let validated = validate_path(&args.path, &config).map_err(|e| ErrorData::internal_error(e, None))?;
        let target = validated.canonical_path;

        if !target.exists() {
            return Err(ErrorData::internal_error(format!("Path not found: {}", args.path), None));
        }

        let meta = fs::metadata(&target).map_err(|e| {
            ErrorData::internal_error(format!("Failed to read metadata: {}", e), None)
        })?;

        let file_type = if meta.is_file() { "file" } else if meta.is_dir() { "directory" } else { "symlink/other" };
        let size = meta.len();
        let modified = meta.modified().ok().map(|t| {
            let dt: chrono::DateTime<chrono::Local> = t.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        }).unwrap_or_else(|| "unknown".to_string());

        let permission = &validated.folder.permission;
        let perm_str = match permission {
            crate::config::Permission::ReadOnly => "read-only",
            crate::config::Permission::ReadWrite => "read-write",
        };

        let mut output = String::new();
        output.push_str(&format!("Path: {}\n", args.path));
        output.push_str(&format!("Type: {}\n", file_type));
        output.push_str(&format!("Size: {} ({} bytes)\n", format_size(size), size));
        output.push_str(&format!("Modified: {}\n", modified));
        output.push_str(&format!("Permission: {}\n", perm_str));

        if meta.is_file() {
            let mime = mime_guess::from_path(&target).first_or_text_plain();
            output.push_str(&format!("MIME type: {}\n", mime));

            // Count lines for text files
            let filename = target.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if is_supported_extension(filename) && !is_binary_file(filename) {
                if let Ok(content) = fs::read_to_string(&target) {
                    output.push_str(&format!("Line count: {}\n", content.lines().count()));
                }
            }
        } else if meta.is_dir() {
            // Count immediate children
            if let Ok(entries) = fs::read_dir(&target) {
                let count = entries.count();
                output.push_str(&format!("Children: {} items\n", count));
            }
        }

        Ok(success_log("get_file_info", "read", Some(&args.path.clone()), "Read file metadata", vec![Content::text(output)]))
    }

    // ────────────────────────────────────────────────────────
    // 11. batch_read — read multiple files at once
    // ────────────────────────────────────────────────────────

    #[tool(description = "Read multiple files in a single call. Returns content for each file or per-file errors. Stops if cumulative size exceeds max_total_size_mb.")]
    async fn batch_read(&self, params: Parameters<BatchReadParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        if args.paths.is_empty() {
            return Err(ErrorData::internal_error("paths array is empty. Provide at least one file path.", None));
        }
        if args.paths.len() > 50 {
            return Err(ErrorData::internal_error("Too many paths (max 50). Split into multiple calls.", None));
        }

        let max_bytes = (args.max_total_size_mb * 1024.0 * 1024.0) as u64;
        let mut total_bytes: u64 = 0;
        let mut results = Vec::new();

        for path_str in &args.paths {
            match validate_path(path_str, &config) {
                Err(e) => {
                    results.push(format!("--- {} ---\nERROR: {}\n", path_str, e));
                    continue;
                }
                Ok(validated) => {
                    let file_path = validated.canonical_path;
                    if !file_path.is_file() {
                        results.push(format!("--- {} ---\nERROR: Not a file\n", path_str));
                        continue;
                    }

                    let meta = match fs::metadata(&file_path) {
                        Ok(m) => m,
                        Err(e) => {
                            results.push(format!("--- {} ---\nERROR: {}\n", path_str, e));
                            continue;
                        }
                    };

                    if total_bytes + meta.len() > max_bytes {
                        results.push(format!(
                            "--- {} ---\nSKIPPED: Would exceed max_total_size_mb ({:.1} MB). Use a separate call.\n",
                            path_str, args.max_total_size_mb
                        ));
                        continue;
                    }

                    let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if is_binary_file(filename) {
                        results.push(format!("--- {} ---\nSKIPPED: Binary file. Use read_file for binary content.\n", path_str));
                        continue;
                    }

                    match fs::read_to_string(&file_path) {
                        Ok(content) => {
                            total_bytes += content.len() as u64;
                            results.push(format!("--- {} ---\n{}\n", path_str, content));
                        }
                        Err(e) => {
                            results.push(format!("--- {} ---\nERROR: {}\n", path_str, e));
                        }
                    }
                }
            }
        }

        let header = format!("Batch read: {} file(s), {}\n\n", args.paths.len(), format_size(total_bytes));

        Ok(success_log("batch_read", "read", Some(&format!("{} paths", args.paths.len())), "Batch read files", vec![Content::text(format!("{}{}", header, results.join("\n")))]))
    }

    // ────────────────────────────────────────────────────────
    // 12. zip_files — create a zip archive
    // ────────────────────────────────────────────────────────

    #[tool(description = "Create a zip archive from one or more files. All source paths must be readable, output path must be writable.")]
    async fn zip_files(&self, params: Parameters<ZipFilesParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        if args.paths.is_empty() {
            return Err(ErrorData::internal_error("paths array is empty.", None));
        }

        let out_validated = validate_writable(&args.output_path, &config)
            .map_err(|e| ErrorData::internal_error(e, None))?;
        let out_path = out_validated.canonical_path;

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ErrorData::internal_error(format!("Failed to create output directory: {}", e), None)
            })?;
        }

        let file = fs::File::create(&out_path).map_err(|e| {
            ErrorData::internal_error(format!("Failed to create zip file: {}", e), None)
        })?;
        let mut zip_writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        let mut file_count = 0u32;

        for path_str in &args.paths {
            let validated = validate_path(path_str, &config)
                .map_err(|e| ErrorData::internal_error(e, None))?;
            let src_path = validated.canonical_path;

            if src_path.is_file() {
                let name = src_path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                zip_writer.start_file(name, options).map_err(|e| {
                    ErrorData::internal_error(format!("Zip error: {}", e), None)
                })?;
                let mut f = fs::File::open(&src_path).map_err(|e| {
                    ErrorData::internal_error(format!("Failed to open {}: {}", path_str, e), None)
                })?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf).map_err(|e| {
                    ErrorData::internal_error(format!("Failed to read {}: {}", path_str, e), None)
                })?;
                zip_writer.write_all(&buf).map_err(|e| {
                    ErrorData::internal_error(format!("Zip write error: {}", e), None)
                })?;
                file_count += 1;
            } else if src_path.is_dir() {
                // Walk directory and add all files
                let walker = walkdir::WalkDir::new(&src_path).into_iter().filter_map(|e| e.ok());
                for entry in walker {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        let rel = entry_path.strip_prefix(&src_path).unwrap_or(entry_path);
                        let name = rel.to_string_lossy().to_string();

                        // Validate each file in sandbox
                        let entry_str = entry_path.to_string_lossy().to_string();
                        if validate_path(&entry_str, &config).is_err() { continue; }

                        zip_writer.start_file(&name, options).map_err(|e| {
                            ErrorData::internal_error(format!("Zip error: {}", e), None)
                        })?;
                        let mut f = fs::File::open(entry_path).map_err(|e| {
                            ErrorData::internal_error(format!("Failed to open: {}", e), None)
                        })?;
                        let mut buf = Vec::new();
                        f.read_to_end(&mut buf).map_err(|e| {
                            ErrorData::internal_error(format!("Failed to read: {}", e), None)
                        })?;
                        zip_writer.write_all(&buf).map_err(|e| {
                            ErrorData::internal_error(format!("Zip write error: {}", e), None)
                        })?;
                        file_count += 1;
                    }
                }
            }
        }

        zip_writer.finish().map_err(|e| {
            ErrorData::internal_error(format!("Failed to finalize zip: {}", e), None)
        })?;

        let zip_size = fs::metadata(&out_path).map(|m| format_size(m.len())).unwrap_or_default();

        Ok(success_log("zip_files", "write", Some(&args.output_path.clone()), "Created zip archive", vec![Content::text(
            format!("Created zip archive: {} ({} files, {})", args.output_path, file_count, zip_size),
        )]))
    }

    // ────────────────────────────────────────────────────────
    // 13. unzip_files — extract a zip archive
    // ────────────────────────────────────────────────────────

    #[tool(description = "Extract a zip archive to a directory. Archive must be readable, destination must be writable.")]
    async fn unzip_files(&self, params: Parameters<UnzipFilesParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let arc_validated = validate_path(&args.archive_path, &config)
            .map_err(|e| ErrorData::internal_error(e, None))?;
        let dst_validated = validate_writable(&args.destination, &config)
            .map_err(|e| ErrorData::internal_error(e, None))?;

        let archive_path = arc_validated.canonical_path;
        let dest_path = dst_validated.canonical_path;

        let file = fs::File::open(&archive_path).map_err(|e| {
            ErrorData::internal_error(format!("Failed to open archive: {}", e), None)
        })?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            ErrorData::internal_error(format!("Invalid zip archive: {}", e), None)
        })?;

        fs::create_dir_all(&dest_path).map_err(|e| {
            ErrorData::internal_error(format!("Failed to create destination: {}", e), None)
        })?;

        let mut extracted = 0u32;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| {
                ErrorData::internal_error(format!("Zip read error: {}", e), None)
            })?;

            let out_path = dest_path.join(entry.mangled_name());

            // Security: ensure extracted path stays within destination
            if !out_path.starts_with(&dest_path) {
                eprintln!("[omnidrive] Skipping suspicious zip entry: {}", entry.name());
                continue;
            }

            if entry.is_dir() {
                fs::create_dir_all(&out_path).ok();
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                let mut outfile = fs::File::create(&out_path).map_err(|e| {
                    ErrorData::internal_error(format!("Failed to create {}: {}", out_path.display(), e), None)
                })?;
                std::io::copy(&mut entry, &mut outfile).map_err(|e| {
                    ErrorData::internal_error(format!("Failed to extract: {}", e), None)
                })?;
                extracted += 1;
            }
        }

        Ok(success_log("unzip_files", "write", Some(&args.destination.clone()), "Extracted zip archive", vec![Content::text(
            format!("Extracted {} files to {}", extracted, args.destination),
        )]))
    }

    // ────────────────────────────────────────────────────────
    // 14. patch_file — targeted search-and-replace editing
    // ────────────────────────────────────────────────────────

    #[tool(description = "Apply targeted edits to a file without rewriting it entirely. Supports search-and-replace (literal or regex) and line-range replacement. Requires Read/Write permission.")]
    async fn patch_file(&self, params: Parameters<PatchFileParams>) -> Result<CallToolResult, ErrorData> {
        let args = params.0;
        let config = self.config.read().await;

        let validated = validate_writable(&args.path, &config)
            .map_err(|e| ErrorData::internal_error(e, None))?;
        let file_path = validated.canonical_path;

        if !file_path.is_file() {
            return Err(ErrorData::internal_error(format!("File not found: {}", args.path), None));
        }

        if args.search_replace.is_empty() && args.line_replace.is_empty() {
            return Err(ErrorData::internal_error(
                "No operations provided. Provide at least one search_replace or line_replace operation.",
                None,
            ));
        }

        let mut content = fs::read_to_string(&file_path).map_err(|e| {
            ErrorData::internal_error(format!("Cannot read file: {}", e), None)
        })?;

        let mut summary = Vec::new();

        // Apply search-and-replace operations
        for (i, op) in args.search_replace.iter().enumerate() {
            if op.regex {
                let re = regex::Regex::new(&op.search).map_err(|e| {
                    ErrorData::internal_error(format!("Invalid regex in operation {}: {}", i + 1, e), None)
                })?;

                let limit = op.count.unwrap_or(0);
                let (new_content, replacements) = if limit > 0 {
                    let mut count = 0usize;
                    let result = re.replace_all(&content, |caps: &regex::Captures| {
                        count += 1;
                        if count <= limit {
                            op.replace.clone()
                        } else {
                            caps[0].to_string()
                        }
                    });
                    (result.to_string(), count.min(limit))
                } else {
                    let matches = re.find_iter(&content).count();
                    let result = re.replace_all(&content, op.replace.as_str());
                    (result.to_string(), matches)
                };

                summary.push(format!("Op {}: regex '{}' → {} replacement(s)", i + 1, op.search, replacements));
                content = new_content;
            } else {
                let limit = op.count.unwrap_or(0);
                let mut count = 0usize;
                let mut new_content = String::new();
                let mut remaining = content.as_str();

                while let Some(pos) = remaining.find(&op.search) {
                    if limit > 0 && count >= limit { break; }
                    new_content.push_str(&remaining[..pos]);
                    new_content.push_str(&op.replace);
                    remaining = &remaining[pos + op.search.len()..];
                    count += 1;
                }
                new_content.push_str(remaining);

                summary.push(format!("Op {}: '{}' → {} replacement(s)", i + 1, op.search, count));
                content = new_content;
            }
        }

        // Apply line-range replacements (on the already-modified content)
        if !args.line_replace.is_empty() {
            let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

            // Sort by start_line descending so we can replace without shifting indices
            let mut line_ops = args.line_replace.clone();
            line_ops.sort_by(|a, b| b.start_line.cmp(&a.start_line));

            for (i, op) in line_ops.iter().enumerate() {
                if op.start_line == 0 || op.end_line == 0 || op.start_line > op.end_line {
                    return Err(ErrorData::internal_error(
                        format!("Invalid line range in line_replace op: {}-{} (1-indexed, start <= end)", op.start_line, op.end_line),
                        None,
                    ));
                }
                let start = (op.start_line - 1).min(lines.len());
                let end = op.end_line.min(lines.len());

                let new_lines: Vec<String> = op.content.lines().map(|l| l.to_string()).collect();
                let removed = end - start;
                lines.splice(start..end, new_lines.clone());

                summary.push(format!(
                    "Line op {}: replaced lines {}-{} ({} lines → {} lines)",
                    i + 1, op.start_line, op.end_line, removed, new_lines.len()
                ));
            }

            content = lines.join("\n");
            // Preserve trailing newline if original had one
            if !content.ends_with('\n') {
                content.push('\n');
            }
        }

        // Write back
        fs::write(&file_path, &content).map_err(|e| {
            ErrorData::internal_error(format!("Failed to write patched file: {}", e), None)
        })?;

        let result = format!(
            "Patched {}\n\n{}\n\nFinal size: {}",
            args.path,
            summary.join("\n"),
            format_size(content.len() as u64),
        );

        Ok(success_log("patch_file", "write", Some(&args.path.clone()), "Patched file contents", vec![Content::text(result)]))
    }
}

// ─── Helper: list_directory recursive ───

impl OmniDriveServer {
    async fn list_directory_recursive(
        &self,
        dir_path: &std::path::Path,
        args: &ListDirectoryParams,
        config: &AppConfig,
    ) -> Result<CallToolResult, ErrorData> {
        let max_depth = args.max_depth.clamp(1, 10);
        let mut entries = Vec::new();

        let walker = walkdir::WalkDir::new(dir_path)
            .max_depth(max_depth)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            let path = entry.path();
            let depth = entry.depth();
            if depth == 0 { continue; } // Skip root

            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let is_dir = path.is_dir();

            if !is_dir && !is_supported_extension(name) { continue; }

            // Check .mcpignore
            let path_str = path.to_string_lossy().to_string();
            if validate_path(&path_str, config).is_err() { continue; }

            let size = if is_dir { 0 } else { entry.metadata().map(|m| m.len()).unwrap_or(0) };
            let indent = "  ".repeat(depth - 1);
            let type_marker = if is_dir { "📁" } else { "📄" };
            let size_str = if is_dir { String::new() } else { format!(" ({})", format_size(size)) };

            entries.push(format!("{}{} {}{}", indent, type_marker, name, size_str));
        }

        // Paginate the flat list of tree entries
        let total_items = entries.len();
        let page_size = args.page_size.clamp(1, 200);
        let page = args.page.max(1);
        let start_idx = (page - 1) * page_size;

        if start_idx >= total_items && total_items > 0 {
            return

        Ok(success_log("list_directory_recursive", "read", Some(&args.path.clone()), "Listed directory recursively", vec![Content::text(format!(
                "Page {} is out of range. Total items: {} ({} pages)",
                page, total_items, (total_items + page_size - 1) / page_size
            ))]));
        }

        let paged: Vec<&String> = entries.iter().skip(start_idx).take(page_size).collect();

        let mut output = String::new();
        output.push_str(&format!("Tree: {} (depth: {}, page {}/{})\n",
            args.path, max_depth, page, (total_items + page_size - 1) / page_size.max(1)));
        output.push_str(&format!("{} items total\n\n", total_items));
        for line in paged {
            output.push_str(line);
            output.push('\n');
        }

        Ok(success_log("list_directory_recursive", "read", Some(&args.path.clone()), "Listed directory recursively", vec![Content::text(output)]))
    }
}

// ─── Helpers ───

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{} B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)) }
    else { format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0)) }
}
