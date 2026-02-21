import { AlertTriangle, FileWarning, X } from "lucide-react";
import type { FolderScanResult } from "../../lib/types";

interface ScanWarningProps {
    result: FolderScanResult;
    onDismiss: () => void;
    disabled?: boolean;
}

export function ScanWarning({ result, onDismiss, disabled = false }: ScanWarningProps) {
    if (result.unsupported_files === 0) {
        return null;
    }

    return (
        <section className="panel panel--warning">
            <div className="scan-warning">
                <div className="scan-warning__icon">
                    <FileWarning size={16} />
                </div>
                <div className="scan-warning__content">
                    <h3>
                        <AlertTriangle size={14} />
                        {result.unsupported_files} unsupported file
                        {result.unsupported_files === 1 ? "" : "s"} detected
                    </h3>
                    <p>
                        {result.supported_files} of {result.total_files} files can be shared
                        with AI agents.
                    </p>
                    {result.unsupported_list.length > 0 ? (
                        <details>
                            <summary>
                                Show unsupported files ({result.unsupported_list.length}
                                {result.unsupported_list.length === 50 ? "+" : ""})
                            </summary>
                            <ul>
                                {result.unsupported_list.map((file, index) => (
                                    <li key={`${file}-${index}`}>{file}</li>
                                ))}
                            </ul>
                        </details>
                    ) : null}
                </div>
                <button
                    type="button"
                    className="icon-button"
                    onClick={onDismiss}
                    aria-label="Dismiss warning"
                    disabled={disabled}
                >
                    <X size={14} />
                </button>
            </div>
        </section>
    );
}
