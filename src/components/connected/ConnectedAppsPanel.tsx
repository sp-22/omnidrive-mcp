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
        <div className="panel apps-panel">
            <div className="panel-header">
                <div className="panel-header-content">
                    <div className="panel-title-wrapper">
                        <Cpu className="panel-title-icon" size={20} />
                        <h2 className="panel-title">Connected AI Agents</h2>
                    </div>
                    <p className="panel-subtitle">Manage agents with real-time access to your shared folders</p>
                </div>
            </div>

            <div className="panel-content">
                <div className="apps-grid">
                    {agents.length > 0 ? (
                        agents.map((agent) => (
                            <div key={agent.name} className={`app-card is-${agent.status}`}>
                                <div className="app-card-header">
                                    <div className="app-icon-placeholder">
                                        <Cpu size={24} />
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

                <div className="setup-section mt-8">
                    <h3 className="setup-title">Quick Setup Snippets</h3>
                    <p className="setup-description">Add this to your MCP client configuration file to connect.</p>

                    <div className="snippet-container">
                        <div className="snippet-header">
                            <span className="snippet-client">Claude Desktop</span>
                            <button
                                type="button"
                                className="btn btn-ghost btn-sm"
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
                </div>
            </div>
        </div>
    );
}
