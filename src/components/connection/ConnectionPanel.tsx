import { useState } from "react";
import { BookOpen, Check, Copy, Terminal } from "lucide-react";
import { useConnectionInfo } from "../../hooks/useConnectionInfo";

const CLIENTS = ["Claude Desktop", "Cursor", "Generic"] as const;

export function ConnectionPanel() {
    const [activeClient, setActiveClient] = useState<string>("Claude Desktop");
    const { serverPath, copied, getConfigSnippet, getConfigPath, copyToClipboard } =
        useConnectionInfo();

    const snippet = getConfigSnippet(activeClient);
    const configPath = getConfigPath(activeClient);

    return (
        <section className="content-section">
            <header className="section-header section-header--stacked">
                <div>
                    <h2>Connections</h2>
                    <p>Configure OmniDrive in your preferred AI client.</p>
                </div>
            </header>

            <div className="settings-stack">
                <section className="settings-group">
                    <div className="panel-title">
                        <Terminal size={16} />
                        <h3>MCP Server Binary</h3>
                    </div>
                    <div className="code-line">
                        <code>{serverPath}</code>
                        <button
                            type="button"
                            className="icon-button"
                            onClick={() => copyToClipboard(serverPath)}
                            aria-label="Copy server path"
                        >
                            {copied ? <Check size={14} /> : <Copy size={14} />}
                        </button>
                    </div>
                </section>

                <section className="settings-group">
                    <div className="panel-title">
                        <BookOpen size={16} />
                        <h3>Setup Guide</h3>
                    </div>

                    <div className="segment-control" role="tablist" aria-label="MCP client">
                        {CLIENTS.map((client) => (
                            <button
                                key={client}
                                type="button"
                                className={`segment-control__button ${activeClient === client ? "is-selected" : ""}`}
                                role="tab"
                                aria-selected={activeClient === client}
                                onClick={() => setActiveClient(client)}
                            >
                                {client}
                            </button>
                        ))}
                    </div>

                    {activeClient !== "Generic" ? (
                        <div className="field-group">
                            <p className="field-label">Config file location</p>
                            <div className="code-line">
                                <code>{configPath}</code>
                            </div>
                        </div>
                    ) : null}

                    <div className="field-group">
                        <p className="field-label">
                            Add this to your {activeClient === "Generic" ? "MCP client" : activeClient} config
                        </p>
                        <div className="code-block">
                            <pre>{snippet}</pre>
                            <button
                                type="button"
                                className="icon-button code-block__copy"
                                onClick={() => copyToClipboard(snippet)}
                                aria-label="Copy config snippet"
                            >
                                {copied ? <Check size={14} /> : <Copy size={14} />}
                            </button>
                        </div>
                    </div>

                    <p className="instruction-text">
                        {activeClient === "Claude Desktop" ? (
                            <>
                                1. Open Claude Desktop Settings, then Developer, then Edit Config.
                                <br />
                                2. Add the snippet above under <code>mcpServers</code>.
                                <br />
                                3. Restart Claude Desktop.
                            </>
                        ) : null}

                        {activeClient === "Cursor" ? (
                            <>
                                1. Open Cursor Settings, then MCP.
                                <br />
                                2. Add a new MCP server.
                                <br />
                                3. Paste the snippet above and restart Cursor.
                            </>
                        ) : null}

                        {activeClient === "Generic" ? (
                            <>
                                Configure your client to launch the binary above using <code>stdio</code>
                                transport. The server reads config from <code>~/.omnidrive/config.json</code>.
                            </>
                        ) : null}
                    </p>
                </section>
            </div>
        </section>
    );
}
