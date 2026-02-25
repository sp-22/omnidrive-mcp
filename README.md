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

## 📥 Download (End Users)

If you just want to use OmniDrive, download the latest installer from:

- [GitHub Releases](https://github.com/sp-22/omnidrive-mcp/releases/latest)

Available assets depend on platform and include installers such as `.dmg` (macOS), `.msi`/`.exe` (Windows), and Linux bundles.

---

## 🔧 Build From Source (Developers)

### Prerequisites
- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Tauri system dependencies for your OS: [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

### Setup
1. Clone the repository:
   ```bash
   git clone https://github.com/sp-22/omnidrive-mcp.git
   cd omnidrive-mcp
   ```

2. Install dependencies:
   ```bash
   npm ci
   ```

3. Run in development:
   ```bash
   npm run tauri dev
   ```

4. Build production bundles:
   ```bash
   npm run tauri build
   ```

Build artifacts are generated under `src-tauri/target/release/bundle/`.

---

## 🤝 Contributing

1. Fork this repo
2. Create a branch
3. Commit and push your changes
4. Open a Pull Request

To download source without Git, use GitHub's **Code -> Download ZIP** on the repository page.

---

## 🚀 Release Process (Maintainers)

This repository includes a GitHub Actions workflow that builds and publishes release assets for macOS, Windows, and Linux whenever a version tag is pushed.

1. Update versions:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`

2. Commit and push:
   ```bash
   git add .
   git commit -m "chore: release v0.x.x"
   git push
   ```

3. Create and push a tag:
   ```bash
   git tag v0.x.x
   git push origin v0.x.x
   ```

4. The workflow will publish binaries to:
   - [Releases](https://github.com/sp-22/omnidrive-mcp/releases)

---

## 📦 Tech Stack

- **Frontend**: React, TypeScript, Vanilla CSS (Premium Glassmorphism)
- **Engine**: Rust (High-performance native sidecar)
- **Framework**: Tauri (Secure system bridging)
- **Protocol**: RMCP (Rust implementation of Model Context Protocol)

---

## 📄 License

OmniDrive is released under the **MIT License**. See [LICENSE](LICENSE) for details.
