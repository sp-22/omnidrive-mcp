import re
import sys

def process():
    path = "/Users/shubham/Desktop/LocalSync/OmniDrive/src-tauri/src/bin/mcp_server/tools.rs"
    with open(path, "r") as f:
        content = f.read()

    helper = """
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
"""
    
    # 1. Add helper at top
    if "fn success_log" not in content:
        content = content.replace(
            "use std::sync::Arc;",
            "use std::sync::Arc;\n" + helper
        )

    tool_info = {
        "list_directory": ("read", "args.path.clone()", '"Listed directory items"'),
        "list_directory_recursive": ("read", "args.path.clone()", '"Listed directory recursively"'),
        "read_file": ("read", "args.path.clone()", '"Read file contents"'),
        "write_file": ("write", "args.path.clone()", '&format!("Wrote file: {}", args.path)'),
        "search_files": ("read", "args.pattern.clone()", '"Searched files"'),
        "grep_content": ("read", "args.root_path.clone()", '&format!("Grepped for {}", args.pattern)'),
        "read_lines": ("read", "args.path.clone()", '"Read file lines"'),
        "move_file": ("delete", "args.source.clone()", '&format!("Moved to {}", args.destination)'),
        "delete_file": ("delete", "args.path.clone()", '"Deleted file/dir"'),
        "copy_file": ("write", "args.destination.clone()", '&format!("Copied from {}", args.source)'),
        "get_file_info": ("read", "args.path.clone()", '"Read file metadata"'),
        "batch_read": ("read", "format!(\"{} paths\", args.paths.len())", '"Batch read files"'),
        "zip_files": ("write", "args.output_path.clone()", '"Created zip archive"'),
        "unzip_files": ("write", "args.destination.clone()", '"Extracted zip archive"'),
        "patch_file": ("write", "args.path.clone()", '"Patched file contents"'),
    }

    for tool, (cat, path_expr, summary_expr) in tool_info.items():
        fn_pattern = r"(async fn " + tool + r"\([^\{]+\{)"
        match = re.search(fn_pattern, content)
        if not match:
            continue
        start_idx = match.end()
        next_fn_idx = content.find("async fn ", start_idx)
        if next_fn_idx == -1:
            next_fn_idx = len(content)
        
        chunk = content[start_idx:next_fn_idx]
        
        # Replace CallToolResult::success( with success_log(tool, cat, Some(&path), summary,
        replacement = f'success_log("{tool}", "{cat}", Some(&{path_expr}), {summary_expr}, '
        new_chunk = chunk.replace("CallToolResult::success(", replacement)
        
        content = content[:start_idx] + new_chunk + content[next_fn_idx:]

    with open(path, "w") as f:
        f.write(content)

if __name__ == "__main__":
    process()
