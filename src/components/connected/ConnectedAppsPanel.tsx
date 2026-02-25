import { useState, useEffect } from "react";
import { Copy, CheckCircle2, ShieldAlert, Cpu } from "lucide-react";
import { getConnectedAgents, getMcpServerPath } from "../../lib/tauri";
import type { ConnectedAgent } from "../../lib/types";

export function ConnectedAppsPanel() {
    const [agents, setAgents] = useState<ConnectedAgent[]>([]);
    const [serverPath, setServerPath] = useState("Loading...");
    const [copiedId, setCopiedId] = useState<string | null>(null);

    useEffect(() => {
        getMcpServerPath()
            .then(setServerPath)
            .catch(console.error);

        const fetchAgents = () => {
            getConnectedAgents()
                .then(setAgents)
                .catch(console.error);
        };

        fetchAgents();
        const interval = setInterval(fetchAgents, 5000);
        return () => clearInterval(interval);
    }, []);

    const handleCopy = (id: string, text: string) => {
        navigator.clipboard.writeText(text);
        setCopiedId(id);
        setTimeout(() => setCopiedId(null), 2000);
    };

    const claudeConfig = `
{
  "mcpServers": {
    "omnidrive": {
      "command": "${serverPath}",
      "args": []
    }
  }
}
  `.trim();

    const getStatusBadge = (status: string) => {
        if (status === "connected") {
            return (
                <span className="badge badge-green">
                    <CheckCircle2 size={12} /> Live Connection
                </span>
            );
        }
        return (
            <span className="badge badge-gray">
                Offline
            </span>
        );
    };

    const formatLastSeen = (isoStr: string) => {
        try {
            const d = new Date(isoStr);
            return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
        } catch {
            return "Unknown";
        }
    };

    return (
        <section className="content-section">
            <header className="section-header section-header--stacked">
                <div>
                    <h2>Connected AI Agents</h2>
                    <p>Manage agents with real-time access to your shared folders.</p>
                </div>
            </header>

            <div className="content-section__body">
                <div className="settings-stack">
                    <section className="settings-group">
                        <h3>Active Connections</h3>
                        <p>Agents detected by OmniDrive in the recent polling window.</p>
                        <div className="apps-grid">
                            {agents.length > 0 ? (
                                agents.map((agent) => (
                                    <div key={agent.name} className={`app-card is-${agent.status}`}>
                                        <div className="app-card-header">
                                            <div className="app-icon-placeholder">
                                                <Cpu size={20} />
                                            </div>
                                            <div className="app-info">
                                                <h3 className="app-name">{agent.name}</h3>
                                                <p className="app-meta">Last seen: {formatLastSeen(agent.last_seen)}</p>
                                            </div>
                                            {getStatusBadge(agent.status)}
                                        </div>
                                    </div>
                                ))
                            ) : (
                                <div className="app-card empty-agents">
                                    <p className="text-secondary">No AI agents connected yet. Connect an agent below to see it appear here.</p>
                                </div>
                            )}
                        </div>
                    </section>

                    <section className="settings-group">
                        <h3>Quick Setup Snippets</h3>
                        <p>Add this to your MCP client configuration file to connect.</p>
                        <div className="snippet-container">
                            <div className="snippet-header">
                                <span className="snippet-client">Claude Desktop</span>
                                <button
                                    type="button"
                                    className="button button--secondary"
                                    onClick={() => handleCopy("claude", claudeConfig)}
                                >
                                    {copiedId === "claude" ? <CheckCircle2 size={14} className="text-green-500" /> : <Copy size={14} />}
                                    {copiedId === "claude" ? "Copied" : "Copy JSON"}
                                </button>
                            </div>
                            <pre className="snippet-code">
                                <code>{claudeConfig}</code>
                            </pre>
                            <p className="snippet-help text-xs text-secondary mt-2">
                                <ShieldAlert size={12} className="inline mr-1" />
                                File path: <code>~/Library/Application Support/Claude/claude_desktop_config.json</code>
                            </p>
                        </div>
                    </section>
                </div>
            </div>
        </section>
    );
}
