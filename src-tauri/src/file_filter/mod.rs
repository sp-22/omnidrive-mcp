pub mod types;

use types::FileCategory;

/// All file extensions considered supported for AI agent consumption
const CODE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "c", "cpp", "h", "hpp",
    "rb", "php", "swift", "kt", "sh", "bat", "ps1", "r", "scala", "lua",
    "dart", "zig", "nim", "ex", "exs", "clj", "hs", "ml", "fs", "cs",
];

const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "csv", "log", "env", "gitignore", "dockerignore", "editorconfig",
];

const DATA_EXTENSIONS: &[&str] = &[
    "json", "yaml", "yml", "toml", "xml", "html", "htm", "css", "scss", "sass",
    "less", "sql", "graphql", "proto", "ini", "cfg", "conf",
];

const DOCUMENT_EXTENSIONS: &[&str] = &["pdf"];

const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico",
];

/// Check if a file extension is supported for sharing with AI agents
pub fn is_supported(extension: &str) -> bool {
    get_file_category(extension) != FileCategory::Unsupported
}

/// Get the category of a file based on its extension
pub fn get_file_category(extension: &str) -> FileCategory {
    let ext = extension.to_lowercase();
    let ext = ext.trim_start_matches('.');

    if CODE_EXTENSIONS.contains(&ext) {
        FileCategory::Code
    } else if TEXT_EXTENSIONS.contains(&ext) {
        FileCategory::Text
    } else if DATA_EXTENSIONS.contains(&ext) {
        FileCategory::Data
    } else if DOCUMENT_EXTENSIONS.contains(&ext) {
        FileCategory::Document
    } else if IMAGE_EXTENSIONS.contains(&ext) {
        FileCategory::Image
    } else {
        FileCategory::Unsupported
    }
}

/// Check if a file at the given path is supported based on its extension.
/// Files without extensions are treated as text (e.g., Makefile, Dockerfile).
pub fn is_file_supported(filename: &str) -> bool {
    // Files without extensions that are commonly text
    let extensionless_supported = [
        "Makefile", "Dockerfile", "Jenkinsfile", "Vagrantfile",
        "Gemfile", "Rakefile", "Procfile", "LICENSE", "README",
        "CHANGELOG", "CONTRIBUTING", "AUTHORS",
    ];

    if let Some(ext) = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
    {
        is_supported(ext)
    } else {
        // Check if it's a known extensionless file
        let basename = std::path::Path::new(filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        extensionless_supported.iter().any(|&name| basename == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_code_extensions() {
        assert!(is_supported("rs"));
        assert!(is_supported("py"));
        assert!(is_supported("js"));
        assert!(is_supported("tsx"));
    }

    #[test]
    fn test_supported_text_extensions() {
        assert!(is_supported("txt"));
        assert!(is_supported("md"));
        assert!(is_supported("csv"));
    }

    #[test]
    fn test_supported_data_extensions() {
        assert!(is_supported("json"));
        assert!(is_supported("yaml"));
        assert!(is_supported("toml"));
    }

    #[test]
    fn test_supported_image_extensions() {
        assert!(is_supported("png"));
        assert!(is_supported("jpg"));
        assert!(is_supported("svg"));
    }

    #[test]
    fn test_unsupported_extensions() {
        assert!(!is_supported("exe"));
        assert!(!is_supported("dll"));
        assert!(!is_supported("zip"));
        assert!(!is_supported("mp4"));
        assert!(!is_supported("pptx"));
        assert!(!is_supported("docx"));
        assert!(!is_supported("xlsx"));
    }

    #[test]
    fn test_case_insensitive() {
        assert!(is_supported("RS"));
        assert!(is_supported("Py"));
        assert!(is_supported("JSON"));
    }

    #[test]
    fn test_extensionless_files() {
        assert!(is_file_supported("Makefile"));
        assert!(is_file_supported("Dockerfile"));
        assert!(is_file_supported("LICENSE"));
        assert!(!is_file_supported("randomfile"));
    }

    #[test]
    fn test_file_category() {
        assert_eq!(get_file_category("rs"), FileCategory::Code);
        assert_eq!(get_file_category("md"), FileCategory::Text);
        assert_eq!(get_file_category("json"), FileCategory::Data);
        assert_eq!(get_file_category("pdf"), FileCategory::Document);
        assert_eq!(get_file_category("png"), FileCategory::Image);
        assert_eq!(get_file_category("exe"), FileCategory::Unsupported);
    }
}
