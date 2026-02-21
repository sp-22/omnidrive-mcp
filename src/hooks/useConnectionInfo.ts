import { useState, useEffect, useCallback } from "react";
import * as api from "../lib/tauri";

export function useConnectionInfo() {
    const [serverPath, setServerPath] = useState<string>("");
    const [copied, setCopied] = useState(false);

    useEffect(() => {
        api.getMcpServerPath().then(setServerPath).catch(() => {
            setServerPath("mcp_server");
        });
    }, []);

    const getConfigSnippet = useCallback((clientName: string) => {
        const snippet: Record<string, object> = {
            "Claude Desktop": {
                mcpServers: {
                    "omnidrive": {
                        command: serverPath,
                        args: [],
                    },
                },
            },
            "Cursor": {
                mcpServers: {
                    "omnidrive": {
                        command: serverPath,
                        args: [],
                    },
                },
            },
            "Antigravity": {
                mcpServers: {
                    "omnidrive": {
                        command: serverPath,
                        args: [],
                    },
                },
            },
            "Generic": {
                command: serverPath,
                args: [],
                transport: "stdio",
            },
        };
        return JSON.stringify(snippet[clientName] || snippet["Generic"], null, 2);
    }, [serverPath]);

    const getConfigPath = useCallback((clientName: string) => {
        const isMac = navigator.userAgent.includes("Mac");
        const isWin = navigator.userAgent.includes("Win");

        const paths: Record<string, string> = {
            "Claude Desktop": isMac
                ? "~/Library/Application Support/Claude/claude_desktop_config.json"
                : isWin
                    ? "%APPDATA%\\Claude\\claude_desktop_config.json"
                    : "~/.config/Claude/claude_desktop_config.json",
            "Cursor": "~/.cursor/mcp.json",
            "Antigravity": "~/.gemini/antigravity/mcp_config.json"
        };
        return paths[clientName] || "See client documentation";
    }, []);

    const copyToClipboard = useCallback(async (text: string) => {
        try {
            await navigator.clipboard.writeText(text);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch {
            // Fallback
            const textarea = document.createElement("textarea");
            textarea.value = text;
            document.body.appendChild(textarea);
            textarea.select();
            document.execCommand("copy");
            document.body.removeChild(textarea);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    }, []);

    return {
        serverPath,
        copied,
        getConfigSnippet,
        getConfigPath,
        copyToClipboard,
    };
}
