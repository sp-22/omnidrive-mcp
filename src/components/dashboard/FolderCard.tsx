import {
    AlertTriangle,
    Folder,
    Lock,
    Trash2,
    Unlock,
} from "lucide-react";
import type { SharedFolder } from "../../lib/types";

interface FolderCardProps {
    folder: SharedFolder;
    onRemove: (path: string) => void;
    onSetPermission: (path: string, permission: SharedFolder["permission"]) => void;
    onToggleEnabled: (path: string, enabled: boolean) => void;
    index: number;
    disabled?: boolean;
}

export function FolderCard({
    folder,
    onRemove,
    onSetPermission,
    onToggleEnabled,
    index,
    disabled = false,
}: FolderCardProps) {
    const folderName = folder.path.split(/[/\\]/).filter(Boolean).pop() || folder.path;
    const isReadWrite = folder.permission === "readwrite";

    return (
        <article
            className={`folder-row ${folder.enabled ? "is-active" : ""}`}
            style={{ animationDelay: `${index * 35}ms` }}
            id={`folder-card-${index}`}
        >
            <div className="folder-row__main">
                <span className={`folder-row__icon ${folder.available ? "" : "is-warning"}`}>
                    <Folder size={17} />
                </span>

                <div className="folder-row__content">
                    <div className="folder-row__title-row">
                        <h3>{folderName}</h3>
                        {!folder.available ? (
                            <span className="status-badge status-badge--danger">
                                <AlertTriangle size={12} />
                                Unavailable
                            </span>
                        ) : null}
                    </div>
                    <p className="folder-row__path">{folder.path}</p>
                    <div className="folder-row__meta">
                        <div className="permission-chips" role="group" aria-label="Access level">
                            <button
                                type="button"
                                className={`permission-chip ${!isReadWrite ? "is-active" : ""}`}
                                onClick={() => onSetPermission(folder.path, "readonly")}
                                disabled={disabled || !isReadWrite}
                            >
                                <Lock size={12} />
                                Read Only
                            </button>
                            <button
                                type="button"
                                className={`permission-chip ${isReadWrite ? "is-active" : ""}`}
                                onClick={() => onSetPermission(folder.path, "readwrite")}
                                disabled={disabled || isReadWrite}
                            >
                                <Unlock size={12} />
                                Read &amp; Write
                            </button>
                        </div>
                        <span className="folder-row__description">
                            {isReadWrite
                                ? "AI can create and modify files"
                                : "AI can only read files"}
                        </span>
                    </div>
                </div>
            </div>

            <div className="folder-row__actions">
                <button
                    type="button"
                    className={`switch ${folder.enabled ? "is-active" : ""}`}
                    onClick={() => onToggleEnabled(folder.path, !folder.enabled)}
                    role="switch"
                    aria-checked={folder.enabled}
                    aria-label={folder.enabled ? "Disable sharing" : "Enable sharing"}
                    disabled={disabled}
                />
                <button
                    type="button"
                    className="icon-button"
                    onClick={() => onRemove(folder.path)}
                    title="Remove folder"
                    aria-label="Remove folder"
                    disabled={disabled}
                >
                    <Trash2 size={15} />
                </button>
            </div>
        </article>
    );
}
