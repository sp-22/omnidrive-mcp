//! Path sandbox — validates all file paths are within allowed folders.
//! Also supports `.mcpignore` files in shared folder roots for pattern-based exclusion.

use std::fs;
use std::io::BufRead;
use std::path::Path;

use crate::config::{AppConfig, Permission, SharedFolder};

/// Result of a sandbox validation
pub struct ValidatedPath {
    pub folder: SharedFolder,
    pub canonical_path: std::path::PathBuf,
}

/// Validate that a path is within an allowed, enabled folder.
/// Returns the matching SharedFolder and the canonicalized path.
pub fn validate_path(path: &str, config: &AppConfig) -> Result<ValidatedPath, String> {
    let target = Path::new(path);
    
    // Convert to absolute path manually to avoid canonicalize() requirement for non-existent files
    let target_abs = if target.is_absolute() {
        target.to_path_buf()
    } else {
        std::env::current_dir().map(|d| d.join(target)).unwrap_or_else(|_| target.to_path_buf())
    };

    let target_str = target_abs.to_string_lossy().to_string();

    // Prevent directory traversal
    if target_str.contains("..") {
        return Err(format!("Access denied: Path traversal characters '..' are not allowed: {}", path));
    }

    for folder in &config.folders {
        if !folder.enabled { continue; }

        // Canonicalize the shared folder path (this MUST exist)
        if let Ok(folder_canonical) = fs::canonicalize(&folder.path) {
            let folder_str = folder_canonical.to_string_lossy().to_string();

            // Check if target starts with folder path
            if target_str.starts_with(&folder_str) {
                let remaining = &target_str[folder_str.len()..];
                if remaining.is_empty() || remaining.starts_with('/') || remaining.starts_with('\\') {
                    // Check .mcpignore patterns
                    if is_ignored(&target_abs, &folder_canonical) {
                        return Err(format!(
                            "Access denied: '{}' is excluded by .mcpignore rules.", path
                        ));
                    }
                    return Ok(ValidatedPath {
                        folder: folder.clone(),
                        canonical_path: target_abs,
                    });
                }
            }
        }
    }

    Err(format!("Access denied: Path '{}' is not within any shared folder.", path))
}

/// Validate that a path is within a writable folder
pub fn validate_writable(path: &str, config: &AppConfig) -> Result<ValidatedPath, String> {
    let validated = validate_path(path, config)?;

    if validated.folder.permission != Permission::ReadWrite {
        return Err(format!(
            "Write access denied: '{}' is in a read-only shared folder. \
             The folder '{}' must be set to Read/Write mode in OmniDrive.",
            path, validated.folder.path
        ));
    }

    Ok(validated)
}

/// Validate that a path exists and is within a writable folder (for destructive ops like delete/move)
pub fn validate_destructive(path: &str, config: &AppConfig) -> Result<ValidatedPath, String> {
    let validated = validate_writable(path, config)?;

    if !validated.canonical_path.exists() {
        return Err(format!(
            "Path not found: '{}'. Cannot perform destructive operation on a non-existent path.",
            path
        ));
    }

    Ok(validated)
}

/// Check if a path should be ignored based on .mcpignore rules in the shared folder root
fn is_ignored(target: &Path, folder_root: &Path) -> bool {
    let ignore_file = folder_root.join(".mcpignore");
    if !ignore_file.exists() {
        return false;
    }

    let file = match fs::File::open(&ignore_file) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let relative = match target.strip_prefix(folder_root) {
        Ok(r) => r.to_string_lossy().to_string(),
        Err(_) => return false,
    };

    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let pattern = line.trim();
        // Skip empty lines and comments
        if pattern.is_empty() || pattern.starts_with('#') {
            continue;
        }

        // Match against the relative path — support glob patterns
        let glob_str = if pattern.contains('/') {
            pattern.to_string()
        } else {
            // Bare name like "node_modules" should match anywhere in the tree  
            format!("**/{}", pattern)
        };

        if let Ok(compiled) = glob::Pattern::new(&glob_str) {
            if compiled.matches(&relative) {
                return true;
            }
            // Also check if any parent directory matches (e.g. "node_modules" ignores all children)
            let with_wildcard = format!("{}/**", glob_str);
            if let Ok(compiled_deep) = glob::Pattern::new(&with_wildcard) {
                if compiled_deep.matches(&relative) {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if a file extension is supported for sharing with AI agents
pub fn is_supported_extension(filename: &str) -> bool {
    let supported_extensions = [
        // Code
        "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "c", "cpp", "h", "hpp",
        "rb", "php", "swift", "kt", "sh", "bat", "ps1", "r", "scala", "lua",
        "dart", "zig", "nim", "ex", "exs", "clj", "hs", "ml", "fs", "cs",
        // Text
        "txt", "md", "csv", "log", "env", "gitignore", "dockerignore",
        // Data
        "json", "yaml", "yml", "toml", "xml", "html", "htm", "css", "scss",
        "less", "sql", "graphql", "proto", "ini", "cfg", "conf",
        // Documents
        "pdf",
        // Images
        "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico",
    ];

    let extensionless_supported = [
        "Makefile", "Dockerfile", "Jenkinsfile", "Vagrantfile",
        "Gemfile", "Rakefile", "Procfile", "LICENSE", "README",
        "CHANGELOG", "CONTRIBUTING", "AUTHORS",
    ];

    if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
        supported_extensions.contains(&ext.to_lowercase().as_str())
    } else {
        let basename = Path::new(filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        extensionless_supported.contains(&basename)
    }
}

/// Determine if a file should be returned as base64 (binary) or text
pub fn is_binary_file(filename: &str) -> bool {
    let binary_extensions = [
        "png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "pdf",
    ];

    if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
        binary_extensions.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}

/// Check if a file is a PDF
pub fn is_pdf(filename: &str) -> bool {
    Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| ext.to_lowercase() == "pdf")
        .unwrap_or(false)
}

/// Check if a file is an image
#[allow(dead_code)]
pub fn is_image(filename: &str) -> bool {
    let image_extensions = ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico"];

    Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| image_extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn test_config() -> AppConfig {
        AppConfig {
            folders: vec![
                SharedFolder {
                    path: "/tmp/test-shared".to_string(),
                    permission: Permission::ReadWrite,
                    enabled: true,
                    available: true,
                },
            ],
            max_file_size_mb: 50,
        }
    }

    #[test]
    fn test_supported_extensions() {
        assert!(is_supported_extension("test.rs"));
        assert!(is_supported_extension("test.py"));
        assert!(is_supported_extension("test.json"));
        assert!(is_supported_extension("test.pdf"));
        assert!(is_supported_extension("test.png"));
    }

    #[test]
    fn test_unsupported_extensions() {
        assert!(!is_supported_extension("test.exe"));
        assert!(!is_supported_extension("test.zip"));
        assert!(!is_supported_extension("test.pptx"));
        assert!(!is_supported_extension("test.mp4"));
    }

    #[test]
    fn test_binary_detection() {
        assert!(is_binary_file("test.png"));
        assert!(is_binary_file("test.pdf"));
        assert!(!is_binary_file("test.rs"));
        assert!(!is_binary_file("test.md"));
    }

    #[test]
    fn test_extensionless_files() {
        assert!(is_supported_extension("Makefile"));
        assert!(is_supported_extension("Dockerfile"));
        assert!(!is_supported_extension("randomname"));
    }
}
