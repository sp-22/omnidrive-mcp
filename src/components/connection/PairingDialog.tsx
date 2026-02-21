import type React from "react";
import { ShieldAlert, Check, X } from "lucide-react";

interface PairingDialogProps {
    origin: string;
    onApprove: (origin: string) => void;
    onDeny: () => void;
}

export const PairingDialog: React.FC<PairingDialogProps> = ({ origin, onApprove, onDeny }) => {
    return (
        <div className="modal-overlay">
            <div className="modal-panel modal-panel--warning">
                <div className="modal-header">
                    <div className="modal-header__icon modal-header__icon--warning">
                        <ShieldAlert size={20} />
                    </div>
                    <div className="modal-header__content">
                        <h3>Connection Request</h3>
                        <p>An external browser application is trying to connect.</p>
                    </div>
                </div>

                <div className="modal-body">
                    <div className="pairing-box">
                        <p className="pairing-box__label">Origin</p>
                        <div className="pairing-box__value">{origin}</div>
                    </div>

                    <p className="modal-description">
                        Only allow this connection if you trust the application. Once allowed, it will have access to all your shared folders.
                    </p>
                </div>

                <div className="modal-footer">
                    <button
                        type="button"
                        className="button button--secondary"
                        onClick={onDeny}
                    >
                        <X size={16} />
                        Block
                    </button>
                    <button
                        type="button"
                        className="button button--primary"
                        onClick={() => onApprove(origin)}
                    >
                        <Check size={16} />
                        Allow Connection
                    </button>
                </div>
            </div>
        </div>
    );
};
