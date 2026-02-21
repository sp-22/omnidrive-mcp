# OmniDrive 🌌

**Securely bridge your local context to AI agents via the Model Context Protocol (MCP).**

OmniDrive is a privacy-first desktop application that allows you to selectively share local folders with AI agents like **Claude Desktop**, **Cursor**, **ChatGPT**, and **Gemini**. By acting as a secure gateway, OmniDrive gives your AI assistants the "eyes" they need to see your codebase, documentation, and data—while keeping you in total control.

---

## 🚀 Why OmniDrive?

Most AI agents struggle with "context blindness"—they can't see the files on your machine without tedious manual uploading. Existing solutions often lack security, exposing your entire home directory or requiring complex CLI setups.

**OmniDrive solves this by providing:**
- **Granular Control**: Share only the specific folders you want.
- **Strict Sandboxing**: AI access is restricted to approved paths; path traversal is impossible.
- **Read/Write Modes**: Choose between read-only or full-access permissions per folder.
- **Privacy First**: No files ever leave your machine. The AI reads them locally via the MCP protocol.

---

## ✨ Core Features

- **Built on MCP**: Full compatibility with the industry-standard Model Context Protocol.
- **Intuitive Desktop UI**: Manage your "sharing bridges" with a clean, macOS-inspired glassmorphic interface.
- **Real-time Activity Log**: Monitor exactly what your AI is doing with your files.
- **Smart File Filters**: Automatically ignores `.git`, `node_modules`, and other noise via `.mcpignore`.
- **SSE Support**: Connect browser-based tools (ChatGPT/Gemini) to your local files over a secure, paired Server-Sent Events (SSE) bridge.
- **Native Performance**: Built with **Rust** and **Tauri** for lightweight, blazing-fast execution.

---

## 🛠️ Use Cases

### 1. Advanced Code Reviews
Point OmniDrive to your project folder and ask Claude to perform a full security audit or refactor multiple files at once.

### 2. Documentation Intelligence
Share your local library of PDFs and Markdown docs. Use your AI assistant to search through your private knowledge base without uploading data to the cloud.

### 3. Data Analysis
Connect folders containing CSV or JSON data. Let AI scripts process and analyze your local datasets directly via the standard file tools.

### 4. Personal Knowledge Base
Bridge your Obsidian or Notion-export folders to your AI agent to talk to your second brain in real-time.

---

## 🔧 Getting Started

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/sp-22/omnidrive-mcp.git
   cd omnidrive-mcp
   ```

2. **Install Dependencies**:
   ```bash
   npm install
   ```

3. **Run Development Mode**:
   ```bash
   npm run tauri dev
   ```

### Connecting to AI Clients

#### Claude Desktop
Add this snippet to your `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "omnidrive": {
      "command": "/path/to/omnidrive-server",
      "args": []
    }
  }
}
```

---

## 🛡️ Security & Privacy

OmniDrive is built with a "zero-trust" approach to its sidecar server:
- **Path Validation**: Every tool call is validated against a whitelist of approved canonical paths.
- **Ignore Rules**: Supports `.mcpignore` files in shared roots to hide sensitive files.
- **Paired Connections**: SSE mode requires explicit manual approval for every browser origin that tries to connect.

---

## 📦 Tech Stack

- **Frontend**: React, TypeScript, Vanilla CSS (Glassmorphism)
- **Backend**: Rust, Tauri
- **Server**: RMCP (Rust Model Context Protocol), Axum (SSE)
- **Icons**: Lucide React

---

## 📄 License

OmniDrive is released under the **MIT License**. See [LICENSE](LICENSE) for details.
