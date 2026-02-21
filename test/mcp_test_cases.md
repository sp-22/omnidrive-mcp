# MCP Server Edge Case Test Cases

This document outlines scenarios and exact prompts you can give to an AI assistant (acting as an MCP client) to test the edge cases and security boundaries of the `mcp-drive` server.

## 1. Sandbox Validation & Path Traversal (Security)
**Goal:** Ensure the server strictly prevents access outside of the defined shared directories, especially using path traversal techniques.

*   **Test Prompt 1 (Path Traversal):**
    > "Please list the directory contents of `/Users/shubham/Desktop/Fun Projects/../` using your `mcp-drive` tool."
    * **Expected Result:** The server blocks the request with an "Access denied: Traversal attempts not allowed" error.

*   **Test Prompt 2 (Accessing an unshared system path):**
    > "Read the file at `/etc/passwd` or `/Users/shubham/.zshrc` using the `mcp-drive` server."
    * **Expected Result:** The server blocks the request with an "Access denied: Path is not within any shared folder" error.

*   **Test Prompt 3 (Tricky overlapping folder names):**
    > "Using `mcp-drive`, create a file in `/Users/shubham/Desktop/Fun Projects 2/test.md`" (assuming only "Fun Projects" is shared).
    * **Expected Result:** The server mathematically resolves paths and blocks it because "Fun Projects 2" is not genuinely within "Fun Projects".

## 2. File Creation & Read-Only Boundaries
**Goal:** Verify that our recent fix to `sandbox.rs` allows creating new files in allowed locations, but blocks writes to read-only paths.

*   **Test Prompt 4 (Creating a file in a non-existent subdirectory):**
    > "Using the `mcp-drive` server, create a new text file at `/Users/shubham/Desktop/Fun Projects/deep/nested/folder/mcp_test.txt` with the text 'Hello nested world'."
    * **Expected Result:** Success. The server should recursively create the `deep/nested/folder` directories and write the file.

*   **Test Prompt 5 (Writing to a Read-Only shared folder):**
    > "Try to write a file called `hack_test.md` into `/Users/shubham/Desktop/Enagager/` using `mcp-drive`."
    * **Expected Result:** The server blocks it with a "Write access denied ... folder must be set to Read/Write mode" error.

## 3. File Types and Size Limits
**Goal:** Validate that the server correctly identifies binary data, unsupported extensions, and files that exceed the `max_file_size_mb` config limit.

*   **Test Prompt 6 (Reading an unsupported file format):**
    > "Attempt to read any `.exe`, `.zip`, or`.mp4` file present in my shared folders using `mcp-drive`."
    * **Expected Result:** Graceful block complaining about an "Unsupported file type".

*   **Test Prompt 7 (Testing PDF/Binary fallback parsing):**
    > "Read the contents of a `.png` or `.pdf` file in `/Users/shubham/Desktop/Fun Projects/` using `mcp-drive`."
    * **Expected Result:** For PNG, it should return a Base64-encoded string. For a PDF, it should extract the raw text (using `pdf-extract`).

## 4. Search Restraints & Pagination
**Goal:** Test edge cases surrounding bounding boxes on searches and massive directory reads.

*   **Test Prompt 8 (Search boundary tests):**
    > "Search for `*.json` files bounded to the root path `/Users/shubham/Desktop/Fun Projects/` using `mcp-drive`."
    * **Expected Result:** The search should only yield JSON files inside that specific folder without leaking into `/Users/shubham/Desktop/Enagager/`.

*   **Test Prompt 9 (Out-of-bounds pagination):**
    > "List the directory contents of `/Users/shubham/Desktop/Enagager/frontend/node_modules` and explicitly ask to get `page: 9999` using `mcp-drive`."
    * **Expected Result:** The server shouldn't crash. It should return a graceful text message saying "Page 9999 is out of range. Total items: X (Y pages)".

---

## 5. `.mcpignore` Enforcement
**Goal:** Verify that `.mcpignore` patterns block access across all tools.

*   **Test Prompt 10 (.mcpignore blocks listing):**
    > "Create a `.mcpignore` file in the root of a shared folder with the line `node_modules`. Then try to list the contents of the `node_modules` directory inside that folder using `mcp-drive`."
    * **Expected Result:** The server returns "Access denied: path is excluded by .mcpignore rules."

*   **Test Prompt 11 (.mcpignore blocks grep_content):**
    > "Use `grep_content` to search for the string `react` inside a shared folder that has `node_modules` in its `.mcpignore`. The pattern should exist inside `node_modules/react/index.js` but not in the user's own source files."
    * **Expected Result:** Zero results — the grep walker should skip all `.mcpignore`-excluded paths.

## 6. Content Search Edge Cases (`grep_content`)
**Goal:** Test regex, case sensitivity, extension filters, and result limits.

*   **Test Prompt 12 (Regex with special characters):**
    > "Use `grep_content` on a shared folder to search for the regex pattern `fn\s+\w+\(` (function signatures in Rust) with `is_regex: true`."
    * **Expected Result:** Returns matches like `sandbox.rs:16:pub fn validate_path(...)` with line numbers.

*   **Test Prompt 13 (Extension filter + case insensitive):**
    > "Use `grep_content` to find `TODO` with `case_insensitive: true` and `include_extensions: [\"rs\", \"ts\"]` in a shared folder."
    * **Expected Result:** Only matches from `.rs` and `.ts` files, including lowercase `todo` / mixed case `Todo`.

*   **Test Prompt 14 (Empty results & invalid regex):**
    > "Use `grep_content` with pattern `xyzzy_nonexistent_string_12345` in a shared folder."
    > "Use `grep_content` with `is_regex: true` and pattern `[invalid(regex` in a shared folder."
    * **Expected Result:** First returns "No matches found". Second returns an actionable error: "Invalid regex '...'".

## 7. Large File Handling (`read_lines`)
**Goal:** Test head, tail, and range reads, and boundary conditions.

*   **Test Prompt 15 (Tail read):**
    > "Use `read_lines` on a long file (e.g. `tools.rs`) with `tail: 10`."
    * **Expected Result:** Returns exactly the last 10 lines, line-numbered, plus the total line count.

*   **Test Prompt 16 (Range exceeding file length):**
    > "Use `read_lines` on a 50-line file with `start_line: 40, end_line: 999`."
    * **Expected Result:** Returns lines 40–50 without error, clipped to the actual file end.

*   **Test Prompt 17 (Default head behavior):**
    > "Use `read_lines` on a 500-line file with no `start_line`, `end_line`, or `tail` arguments."
    * **Expected Result:** Returns the first 100 lines by default.

## 8. File Manipulation Edge Cases (CRUD)
**Goal:** Test destructive operations and their safety mechanisms.

*   **Test Prompt 18 (Delete non-empty directory):**
    > "Use `delete_file` on a directory that contains files."
    * **Expected Result:** Error: "Failed to delete directory: ... Only empty directories can be deleted."

*   **Test Prompt 19 (Move to existing destination):**
    > "Use `move_file` where the destination already exists as a file."
    * **Expected Result:** Error: "Destination already exists: ... Delete it first or choose a different name."

*   **Test Prompt 20 (Delete from read-only folder):**
    > "Use `delete_file` on a file inside a read-only shared folder."
    * **Expected Result:** "Write access denied" error — destructive ops require ReadWrite permission.

*   **Test Prompt 21 (Copy preserves source):**
    > "Use `copy_file` to copy a file, then use `read_file` on both source and destination. Verify they match."
    * **Expected Result:** Both files return identical content. Source is not removed.

## 9. Metadata Without Content (`get_file_info`)
**Goal:** Verify metadata is returned without wasting tokens on file content.

*   **Test Prompt 22 (Info on text file):**
    > "Use `get_file_info` on `tools.rs` and verify it returns size, modification date, MIME type, and line count without the file content."
    * **Expected Result:** Output includes `Type: file`, `Size: ~X KB`, `Modified: YYYY-MM-DD ...`, `MIME type: text/...`, `Line count: NNN`. No actual code content.

*   **Test Prompt 23 (Info on directory):**
    > "Use `get_file_info` on a directory path."
    * **Expected Result:** Returns `Type: directory`, `Children: N items`, no line count or MIME.

## 10. Bulk Operations (`batch_read`)
**Goal:** Test multi-file reads and size limit enforcement.

*   **Test Prompt 24 (Batch with mixed valid/invalid paths):**
    > "Use `batch_read` with paths: a valid `.rs` file, a path outside the sandbox, a non-existent file, and a binary `.png` file."
    * **Expected Result:** Returns content for the valid file, `ERROR: Access denied` for the out-of-sandbox path, `ERROR: Not a file` or similar for non-existent, `SKIPPED: Binary file` for the PNG.

*   **Test Prompt 25 (Batch exceeding max size):**
    > "Use `batch_read` with `max_total_size_mb: 0.001` (1 KB) on two files that are each >1 KB."
    * **Expected Result:** First file content is returned, second is `SKIPPED: Would exceed max_total_size_mb`.

## 11. Patch File Edge Cases
**Goal:** Test partial editing precision and failure modes.

*   **Test Prompt 26 (Search-and-replace with count limit):**
    > "Use `patch_file` on a file containing the word 'test' 5 times, with `search: 'test', replace: 'TEST', count: 2`."
    * **Expected Result:** Only the first 2 occurrences are replaced. Summary says "2 replacement(s)".

*   **Test Prompt 27 (Line-range replacement):**
    > "Use `patch_file` with `line_replace` to replace lines 3-5 of a 10-line file with a single line 'REPLACED'. Then use `read_lines` to verify the result."
    * **Expected Result:** File now has 8 lines. Lines 1-2 unchanged, line 3 is "REPLACED", former lines 6-10 are now lines 4-8.

*   **Test Prompt 28 (Empty operations error):**
    > "Use `patch_file` with empty `search_replace` and `line_replace` arrays."
    * **Expected Result:** Error: "No operations provided."

## 12. Archive Operations (`zip_files` / `unzip_files`)
**Goal:** Test zip creation, extraction, and security constraints.

*   **Test Prompt 29 (Zip and unzip round-trip):**
    > "Use `zip_files` to archive 3 text files into `test_archive.zip`. Then use `unzip_files` to extract it to a new directory. Finally use `batch_read` on the extracted files to verify content matches."
    * **Expected Result:** Zip creation succeeds with file count and size. Extraction succeeds. Batch read confirms identical content.

*   **Test Prompt 30 (Zip output to read-only folder):**
    > "Use `zip_files` with `output_path` pointing to a read-only shared folder."
    * **Expected Result:** "Write access denied" error.

---

## 13. 🔥 Multi-Tool Workflow Tests
**Goal:** Test complex scenarios that require chaining multiple tools in sequence, verifying the server behaves cohesively across operations.

*   **Test Prompt 31 (Refactoring workflow: grep → read_lines → patch → verify):**
    > "I need to rename a function in my codebase. First, use `grep_content` to find all occurrences of `format_size` across the shared folder. Then use `read_lines` to view 5 lines of context around one of the matches. Then use `patch_file` to rename `format_size` to `humanize_bytes` in `tools.rs`. Finally use `grep_content` again to confirm no remaining `format_size` references in that file."
    * **Expected Result:** grep finds multiple hits → read_lines shows context → patch replaces all occurrences → second grep finds zero matches in that file. (⚠️ Revert after test!)

*   **Test Prompt 32 (Backup + edit + verify workflow: copy → patch → read_lines):**
    > "Copy `sandbox.rs` to `sandbox.rs.bak` using `copy_file`. Then use `patch_file` to add a comment `// PATCHED` at the top by replacing line 1. Then use `read_lines` with `tail: 1` on the backup to confirm it still has the original first line. Finally use `read_lines` on the modified file to confirm line 1 is `// PATCHED`."
    * **Expected Result:** Copy succeeds → patch modifies original → backup retains old content → modified file has new content. Proves non-destructive backup workflow.

*   **Test Prompt 33 (Project scaffold workflow: write → batch_read → zip):**
    > "Create 3 files using `write_file`: `project/README.md`, `project/src/main.rs`, and `project/Cargo.toml` with appropriate starter content. Then use `list_directory` with `recursive: true` to show the tree. Then use `batch_read` to read all 3 files in one call. Finally use `zip_files` to archive the `project/` directory."
    * **Expected Result:** Files created → tree shows correct structure → batch_read returns all 3 contents → zip archive created with file count = 3.

*   **Test Prompt 34 (Audit workflow: list_recursive → get_file_info → grep → batch_read):**
    > "List the entire project tree of a shared folder using `list_directory` with `recursive: true, max_depth: 4`. Then pick the 3 largest `.rs` files from the listing and use `get_file_info` on each. Then use `grep_content` to find any `unwrap()` calls across all Rust files. Finally use `batch_read` on the files that have `unwrap()` to review them."
    * **Expected Result:** Tree gives full picture → file info shows sizes/line counts → grep finds unwrap locations → batch_read pulls the exact files for review. This is a realistic code audit flow.

*   **Test Prompt 35 (Archive + cleanup workflow: zip → delete → unzip → verify):**
    > "First use `write_file` to create `temp/a.txt` and `temp/b.txt`. Use `zip_files` to archive the `temp/` directory into `temp_backup.zip`. Then use `delete_file` on both text files, then delete the empty `temp/` directory. Finally use `unzip_files` to restore from `temp_backup.zip` into `temp_restored/` and use `list_directory` to verify the files are back."
    * **Expected Result:** Create → zip → delete files → delete empty dir → unzip → listing shows restored files. Full lifecycle test.

*   **Test Prompt 36 (Cross-tool error propagation: grep on ignored → move to read-only → batch with oversized):**
    > "Do all of these in sequence: (1) `grep_content` searching inside a path that is excluded by `.mcpignore` — expect 0 results. (2) `move_file` trying to move a file into a read-only folder — expect write denied. (3) `batch_read` with `max_total_size_mb: 0.0001` on a normal file — expect it to be skipped for size. Confirm each operation gives a clear, actionable error."
    * **Expected Result:** Each tool fails gracefully with distinct, helpful error messages — no crashes, no ambiguous errors.

## Note on Testing
If you run these prompts right now and receive an `EOF` error, it signifies that the connection to the `mcp-drive` background process dropped (the binary panicked or the local IDE agent reset the connection). Once the connection is stable, these exact strings are what you feed an AI agent to aggressively vet the Rust codebase!

**After running destructive tests (31, 35), revert any changes to avoid polluting the real codebase.**
