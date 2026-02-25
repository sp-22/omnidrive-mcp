import { useRef, useEffect } from "react";
import { Database, Edit, Trash2, Eye, Server } from "lucide-react";
import { useActivityLog } from "../../hooks/useActivityLog";
import type { ActivityCategory } from "../../lib/types";

export function ActivityLogPanel() {
    const { entries, activeCategory, setCategory, loading, clear } = useActivityLog();
    const listRef = useRef<HTMLDivElement>(null);

    // Auto-scroll to bottom when new entries arrive
    useEffect(() => {
        if (listRef.current) {
            listRef.current.scrollTop = listRef.current.scrollHeight;
        }
    }, [entries]);

    const categories: { id: ActivityCategory; label: string }[] = [
        { id: "all", label: "All Activity" },
        { id: "read", label: "Reads" },
        { id: "write", label: "Writes" },
        { id: "delete", label: "Deletes" },
    ];

    const getCategoryIcon = (category: string) => {
        switch (category) {
            case "read": return <Eye size={12} className="icon-read" />;
            case "write": return <Edit size={12} className="icon-write" />;
            case "delete": return <Trash2 size={12} className="icon-delete" />;
            default: return <Server size={12} className="icon-system" />;
        }
    };

    const getCategoryTheme = (category: string) => {
        switch (category) {
            case "read": return "badge-blue";
            case "write": return "badge-green";
            case "delete": return "badge-red";
            default: return "badge-gray";
        }
    };

    const formatTime = (isoString: string) => {
        try {
            const d = new Date(isoString);
            return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
        } catch {
            return isoString;
        }
    };

    return (
        <section className="content-section content-section--activity">
            <header className="section-header">
                <div>
                    <h2>Agent Activity Log</h2>
                    <p>Real-time telemetry of all MCP AI agent operations.</p>
                </div>
                <button
                    type="button"
                    onClick={clear}
                    className="button button--secondary"
                    disabled={entries.length === 0}
                >
                    Clear Logs
                </button>
            </header>

            <div className="content-section__body">
                <div className="activity-toolbar">
                    <div className="segment-control activity-filters">
                        {categories.map((cat) => (
                            <button
                                key={cat.id}
                                className={`segment-btn ${activeCategory === cat.id ? "is-active" : ""}`}
                                onClick={() => setCategory(cat.id)}
                            >
                                {cat.label}
                            </button>
                        ))}
                    </div>
                </div>

                <div className="activity-log-container">
                    {entries.length === 0 ? (
                        <div className="empty-state">
                            <Database className="empty-state-icon" size={32} />
                            <h3 className="empty-state-title">No Activity Yet</h3>
                            <p className="empty-state-text">
                                {activeCategory === "all"
                                    ? "Waiting for connected AI agents to perform operations..."
                                    : `No ${activeCategory} operations recorded recently.`}
                            </p>
                        </div>
                    ) : (
                        <div className="activity-log" ref={listRef}>
                            {entries.map((entry) => (
                                <div key={entry.id} className="activity-row">
                                    <div className="activity-time">{formatTime(entry.timestamp)}</div>
                                    <div className={`activity-badge ${getCategoryTheme(entry.category)}`}>
                                        {getCategoryIcon(entry.category)}
                                        <span>{entry.category.toUpperCase()}</span>
                                    </div>
                                    <div className="activity-details">
                                        <span className="activity-agent">{entry.agent}</span>
                                        <span className="activity-tool">{entry.tool}</span>
                                        <span className="activity-summary">{entry.summary}</span>
                                    </div>
                                </div>
                            ))}
                            {loading && <div className="activity-loading">Polling for updates...</div>}
                        </div>
                    )}
                </div>
            </div>
        </section>
    );
}
