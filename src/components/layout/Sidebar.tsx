import type { ReactNode } from "react";
import { Plus } from "lucide-react";
import type { ServerUiState } from "../../lib/types";

export interface SidebarItemConfig<T extends string> {
    id: T;
    label: string;
    icon: ReactNode;
    badge?: string;
}

interface SidebarProps<T extends string> {
    items: SidebarItemConfig<T>[];
    active: T;
    onSelect: (item: T) => void;
    onQuickAction: () => void;
    actionsDisabled: boolean;
    serverUiState: ServerUiState;
    activeFolders: number;
    onToggleServer: () => void;
}

export function Sidebar<T extends string>({
    items,
    active,
    onSelect,
    onQuickAction,
    actionsDisabled,
    serverUiState,
    activeFolders,
    onToggleServer,
}: SidebarProps<T>) {
    return (
        <div className="sidebar">
            <div className="sidebar__content">
                <section className="sidebar__header">
                    <button
                        type="button"
                        className="sidebar__quick-button"
                        onClick={onQuickAction}
                        disabled={actionsDisabled}
                    >
                        <Plus size={14} />
                        Connect Folder
                    </button>
                </section>

                <section className="sidebar__section" aria-label="Places">
                    <p className="sidebar__section-label">Places</p>
                    <nav className="sidebar__nav" aria-label="Primary">
                        {items.map((item) => {
                            const isActive = active === item.id;

                            return (
                                <button
                                    key={item.id}
                                    type="button"
                                    className={`sidebar__item ${isActive ? "is-active" : ""}`}
                                    onClick={() => onSelect(item.id)}
                                >
                                    <span className="sidebar__icon">{item.icon}</span>
                                    <span className="sidebar__label">{item.label}</span>
                                    {item.badge ? <span className="sidebar__badge">{item.badge}</span> : null}
                                </button>
                            );
                        })}
                    </nav>
                </section>
            </div>

            <div className="sidebar__footer">
                <div className={`sidebar__server-pill ${serverUiState === "active" ? "is-live" : "is-dormant"}`}>
                    <span className="sidebar__server-dot" />
                    <div>
                        <p>Omni Server</p>
                        <span>
                            {serverUiState === "active" ? "Active" : "Dormant"} · {activeFolders} folder
                            {activeFolders === 1 ? "" : "s"}
                        </span>
                    </div>
                    <button
                        type="button"
                        className={`switch server-switch ${serverUiState === "active" ? "is-active" : ""}`}
                        onClick={onToggleServer}
                        role="switch"
                        aria-checked={serverUiState === "active"}
                        aria-label="Toggle MCP server UI state"
                    />
                </div>
            </div>
        </div>
    );
}
