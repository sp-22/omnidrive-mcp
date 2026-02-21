# OmniDrive 🌌

**The secure, local-first gateway that bridges your personal files to any AI Agent.**

OmniDrive is a "Plug & Play" desktop application that allows you to selectively share local folders with AI agents like **Claude Desktop**, **Cursor**, **ChatGPT**, and **Gemini**. Built on the Model Context Protocol (MCP), it gives your AI assistants the "eyes" they need to see your documents, spreadsheets, and codebase—while keeping you in absolute control.

---

## 🚀 Why OmniDrive?

Most AI agents are currently "context blind"—they don't know what's on your machine unless you manually upload files. Existing solutions are either complex terminal tools or insecure "all-access" scripts.

**OmniDrive solves this by providing:**
- **Zero Configuration**: A clean, beautiful UI replaces complex JSON edits. Just drag, drop, and share.
- **Security-First (Zero Trust)**: AI access is mathematically restricted to approved paths. Path traversal (`../`) is impossible.
- **Granular Permissions**: Set folders to **Read-Only** (safe browsing) or **Read/Write** (autonomous editing) on the fly.
- **Live Activity Streaming**: See every file read, every search, and every edit the AI performs in real-time.

---

## ✨ Key Capabilities

- **Universal Compatibility**: Works with any LLM client that supports the MCP standard.
- **Plug & Play**: Add folders in seconds. No terminal commands, no environment variables.
- **Total Transparency**: A dedicated **Live Activity Log** window shows you exactly what the AI is "thinking" about your files.
- **Privacy by Design**: No files are uploaded to OmniDrive servers. Everything happens over a local process pipe or a secure local SSE bridge.
- **Smart Filtering**: Respects `.mcpignore` files to keep noise like `node_modules` out of the AI's view.

---

## 🛠️ Use Cases (Beyond Coding)

While OmniDrive is a must-have for developers, it is designed for everyone:

### 📚 Research & Analysis
Share your local library of PDFs and research notes. Ask your AI to synthesize trends across dozens of documents without ever hitting "upload."

### 📂 Administrative Automation
Give the AI access to your spreadsheets and invoices. Ask it to generate summaries, identify anomalies, or prepare monthly reports.

### 🧠 Personal Knowledge Base
Bridge your Obsidian or Markdown notes to their AI agent. Talk to your "Second Brain" to find connections between your thoughts in real-time.

### 💻 Advanced Software Engineering
Enable the AI to proactively fix bugs, refactor architecture, and verify changes across your entire repository with full Read/Write power.

---

## 🛡️ Security Architecture

OmniDrive is built with a "Privacy First" philosophy:
- **Canonical Path Validation**: Every request is verified against a secure whitelist of user-approved absolute paths.
- **Sandbox Isolation**: Even if an LLM is "hallucinating" or being malicious, it cannot escape the folders you have explicitly shared.
- **Paired Web Connections**: SSE (for browser agents) is locked to 127.0.0.1 and requires a secure handshake to prevent unauthorized web access.

---

## 🔧 Installation

### Prerequisites
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/) (for building the native sidecar)

### Setup
1. **Clone & Install**:
   ```bash
   git clone https://github.com/sp-22/omnidrive-mcp.git
   cd omnidrive-mcp
   npm install
   ```

2. **Run in Development**:
   ```bash
   npm run tauri dev
   ```

3. **Build Production App**:
   ```bash
   npm run tauri build
   ```

---

## 📦 Tech Stack

- **Frontend**: React, TypeScript, Vanilla CSS (Premium Glassmorphism)
- **Engine**: Rust (High-performance native sidecar)
- **Framework**: Tauri (Secure system bridging)
- **Protocol**: RMCP (Rust implementation of Model Context Protocol)

---

## 📄 License

OmniDrive is released under the **MIT License**. See [LICENSE](LICENSE) for details.
