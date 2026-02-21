import { useEffect, useMemo, useRef, useState } from "react";
import {
  Activity,
  AlertCircle,
  FolderOpen,
  FolderPlus,
  Link2,
  Settings,
  X,
} from "lucide-react";
import { useFolders } from "./hooks/useFolders";
import { useTheme } from "./hooks/useTheme";
import { FolderCard } from "./components/dashboard/FolderCard";
import { ScanWarning } from "./components/dashboard/ScanWarning";
import { AppShell } from "./components/layout/AppShell";
import { Sidebar, type SidebarItemConfig } from "./components/layout/Sidebar";
import { ConnectedAppsPanel } from "./components/connected/ConnectedAppsPanel";
import { ActivityLogPanel } from "./components/activity/ActivityLogPanel";
import { PairingDialog } from "./components/connection/PairingDialog";
import { useActivityLog } from "./hooks/useActivityLog";
import type {
  AppTab,
  ResolvedTheme,
  ServerUiState,
  SharedFolder,
  ThemePreference,
} from "./lib/types";
import { getSseStatus, startSseMode, stopSseMode, approveOrigin, getApprovedOrigins, revokeOrigin } from "./lib/tauri";
import "./index.css";

function App() {
  const [activeTab, setActiveTab] = useState<AppTab>("folders");
  const [serverUiState, setServerUiState] = useState<ServerUiState>("active");
  const [isWaking, setIsWaking] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const wakeTimerRef = useRef<number | null>(null);
  const toastTimerRef = useRef<number | null>(null);
  const {
    folders,
    loading,
    error,
    scanResult,
    showScanWarning,
    addFolder,
    removeFolder,
    setPermission,
    toggleEnabled,
    dismissScanWarning,
    setError,
  } = useFolders();
  const { entries } = useActivityLog();
  const { preference, resolvedTheme, setPreference } = useTheme();

  const [pendingOrigin, setPendingOrigin] = useState<string | null>(null);

  useEffect(() => {
    // Scan recent logs for blocked SSE connections
    const securityLogs = entries.filter(e => e.category === "security");
    if (securityLogs.length > 0) {
      const match = securityLogs[0].summary.match(/Blocked connection attempt from unapproved origin: (.+)/);
      if (match) {
        const origin = match[1];
        // We only show the modal if we aren't already dealing with it
        if (pendingOrigin !== origin) {
          setPendingOrigin(origin);
        }
      }
    }
  }, [entries, pendingOrigin]);

  const handleApprovePairing = async (origin: string) => {
    try {
      await approveOrigin(origin);
      setPendingOrigin(null);
      pushToast(`Approved ${origin}. The browser tool can now connect.`);
    } catch (err) {
      console.error("Failed to approve pairing:", err);
    }
  };

  const activeFolders = useMemo(
    () => folders.filter((folder) => folder.available && folder.enabled).length,
    [folders],
  );

  const actionsDisabled = serverUiState === "dormant" || isWaking;

  const navItems = useMemo<SidebarItemConfig<AppTab>[]>(
    () => [
      {
        id: "folders",
        label: "Shared Folders",
        icon: <FolderOpen size={16} />,
        badge: folders.length > 0 ? String(folders.length) : undefined,
      },
      {
        id: "connectedApps",
        label: "Connected Apps",
        icon: <Link2 size={16} />,
      },
      {
        id: "activityLog",
        label: "Activity Log",
        icon: <Activity size={16} />,
      },
      {
        id: "settings",
        label: "Settings",
        icon: <Settings size={16} />,
      },
    ],
    [folders.length],
  );

  const pushToast = (message: string) => {
    setToastMessage(message);
    if (toastTimerRef.current) {
      window.clearTimeout(toastTimerRef.current);
    }
    toastTimerRef.current = window.setTimeout(() => {
      setToastMessage(null);
    }, 3200);
  };

  const toggleServerUiState = () => {
    if (wakeTimerRef.current) {
      window.clearTimeout(wakeTimerRef.current);
      wakeTimerRef.current = null;
    }

    if (serverUiState === "active") {
      setServerUiState("dormant");
      setIsWaking(false);
      pushToast("All file bridges disconnected. LLMs can no longer access your data.");
      return;
    }

    setServerUiState("active");
    setIsWaking(true);
    wakeTimerRef.current = window.setTimeout(() => {
      setIsWaking(false);
      wakeTimerRef.current = null;
    }, 500);
  };

  useEffect(() => {
    return () => {
      if (wakeTimerRef.current) {
        window.clearTimeout(wakeTimerRef.current);
      }
      if (toastTimerRef.current) {
        window.clearTimeout(toastTimerRef.current);
      }
    };
  }, []);

  return (
    <AppShell
      sidebar={
        <Sidebar
          items={navItems}
          active={activeTab}
          onSelect={setActiveTab}
          onQuickAction={addFolder}
          actionsDisabled={actionsDisabled}
          serverUiState={serverUiState}
          activeFolders={activeFolders}
          onToggleServer={toggleServerUiState}
        />
      }
    >
      <div className="content-shell">
        <div className={`content-stack ${serverUiState === "dormant" ? "is-dormant" : ""}`}>
          {error ? (
            <div className="panel panel--danger">
              <div className="inline-row">
                <AlertCircle size={16} />
                <p>{error}</p>
              </div>
              <button
                type="button"
                className="icon-button"
                onClick={() => setError(null)}
                aria-label="Dismiss error"
                disabled={actionsDisabled}
              >
                <X size={14} />
              </button>
            </div>
          ) : null}

          {isWaking ? (
            <SkeletonPanel />
          ) : (
            <>
              {activeTab === "folders" ? (
                <FoldersTab
                  folders={folders}
                  loading={loading}
                  scanResult={scanResult}
                  showScanWarning={showScanWarning}
                  onAddFolder={addFolder}
                  onRemoveFolder={removeFolder}
                  onSetPermission={setPermission}
                  onToggleEnabled={toggleEnabled}
                  onDismissScanWarning={dismissScanWarning}
                  actionsDisabled={actionsDisabled}
                />
              ) : null}

              {activeTab === "connectedApps" ? <ConnectedAppsPanel /> : null}

              {activeTab === "activityLog" ? <ActivityLogPanel /> : null}

              {activeTab === "settings" ? (
                <SettingsTab
                  themePreference={preference}
                  resolvedTheme={resolvedTheme}
                  setThemePreference={setPreference}
                  actionsDisabled={actionsDisabled}
                />
              ) : null}
            </>
          )}
        </div>

        {toastMessage ? (
          <div className="toast toast--visible" role="status" aria-live="polite">
            {toastMessage}
          </div>
        ) : null}

        {pendingOrigin && (
          <PairingDialog
            origin={pendingOrigin}
            onApprove={handleApprovePairing}
            onDeny={() => setPendingOrigin(null)}
          />
        )}
      </div>
    </AppShell>
  );
}

interface FoldersTabProps {
  folders: SharedFolder[];
  loading: boolean;
  scanResult: {
    total_files: number;
    supported_files: number;
    unsupported_files: number;
    unsupported_list: string[];
  } | null;
  showScanWarning: boolean;
  onAddFolder: () => void;
  onRemoveFolder: (path: string) => void;
  onSetPermission: (path: string, permission: SharedFolder["permission"]) => void;
  onToggleEnabled: (path: string, enabled: boolean) => void;
  onDismissScanWarning: () => void;
  actionsDisabled: boolean;
}

function FoldersTab({
  folders,
  loading,
  scanResult,
  showScanWarning,
  onAddFolder,
  onRemoveFolder,
  onSetPermission,
  onToggleEnabled,
  onDismissScanWarning,
  actionsDisabled,
}: FoldersTabProps) {
  return (
    <section className="content-section">
      <header className="section-header">
        <div>
          <h2>Shared Folders</h2>
          <p>Folders accessible to AI agents through MCP.</p>
        </div>
        <button
          type="button"
          className="button button--primary"
          onClick={onAddFolder}
          disabled={actionsDisabled}
        >
          <FolderPlus size={16} />
          Connect Folder
        </button>
      </header>

      {showScanWarning && scanResult ? (
        <ScanWarning
          result={scanResult}
          onDismiss={onDismissScanWarning}
          disabled={actionsDisabled}
        />
      ) : null}

      {loading ? (
        <div className="panel loading-state">Loading shared folders…</div>
      ) : null}

      {!loading && folders.length === 0 ? (
        <EmptyState onAddFolder={onAddFolder} actionsDisabled={actionsDisabled} />
      ) : null}

      {!loading && folders.length > 0 ? (
        <div className="folder-grid">
          {folders.map((folder, index) => (
            <FolderCard
              key={folder.path}
              folder={folder}
              index={index}
              onRemove={onRemoveFolder}
              onSetPermission={onSetPermission}
              onToggleEnabled={onToggleEnabled}
              disabled={actionsDisabled}
            />
          ))}
        </div>
      ) : null}
    </section>
  );
}

function EmptyState({
  onAddFolder,
  actionsDisabled,
}: {
  onAddFolder: () => void;
  actionsDisabled: boolean;
}) {
  return (
    <div className="panel empty-state">
      <div className="empty-state__icon">
        <FolderPlus size={24} />
      </div>
      <h3>No bridges built yet.</h3>
      <p>
        Select a folder to start sharing context with your AI. Your files never leave
        your computer.
      </p>
      <button
        type="button"
        className="button button--primary"
        onClick={onAddFolder}
        disabled={actionsDisabled}
      >
        <FolderPlus size={16} />
        Add First Folder
      </button>
    </div>
  );
}

interface SettingsTabProps {
  themePreference: ThemePreference;
  resolvedTheme: ResolvedTheme;
  setThemePreference: (value: ThemePreference) => void;
  actionsDisabled: boolean;
}

function SettingsTab({
  themePreference,
  resolvedTheme,
  setThemePreference,
  actionsDisabled,
}: SettingsTabProps) {
  const [maxFileSizeMb, setMaxFileSizeMb] = useState(50);
  const appearanceOptions: ThemePreference[] = ["system", "light", "dark"];

  // SSE State
  const [sseRunning, setSseRunning] = useState(false);
  const [ssePort, setSsePort] = useState(3199);
  const [sseUrl, setSseUrl] = useState<string | null>(null);
  const [approvedOrigins, setApprovedOrigins] = useState<string[]>([]);

  useEffect(() => {
    getSseStatus().then((status) => {
      setSseRunning(status.running);
      if (status.port > 0) setSsePort(status.port);
      setSseUrl(status.url);
    }).catch(console.error);

    getApprovedOrigins().then(setApprovedOrigins).catch(console.error);
  }, []);

  const toggleSse = async () => {
    try {
      if (sseRunning) {
        const status = await stopSseMode();
        setSseRunning(status.running);
        setSseUrl(status.url);
      } else {
        const origins = ["https://chatgpt.com", "https://gemini.google.com", "https://claude.ai", "https://aistudio.google.com"];
        const status = await startSseMode(ssePort, origins);
        setSseRunning(status.running);
        setSseUrl(status.url);
      }
    } catch (err) {
      console.error("Failed to toggle SSE mode:", err);
    }
  };

  const handleRevoke = async (origin: string) => {
    try {
      await revokeOrigin(origin);
      const updated = await getApprovedOrigins();
      setApprovedOrigins(updated);
    } catch (err) {
      console.error("Failed to revoke origin:", err);
    }
  };

  return (
    <section className="content-section">
      <header className="section-header section-header--stacked">
        <div>
          <h2>Settings</h2>
          <p>Configure OmniDrive behavior.</p>
        </div>
      </header>

      <div className="settings-stack">
        <section className="settings-group">
          <h3>Appearance</h3>
          <p>Use system appearance or override it for this app.</p>
          <div className="segment-control" role="radiogroup" aria-label="Appearance">
            {appearanceOptions.map((option) => (
              <button
                key={option}
                type="button"
                className={`segment-control__button ${themePreference === option ? "is-selected" : ""}`}
                role="radio"
                aria-checked={themePreference === option}
                onClick={() => setThemePreference(option)}
                disabled={actionsDisabled}
              >
                {option.charAt(0).toUpperCase() + option.slice(1)}
              </button>
            ))}
          </div>
          <span className="settings-note">Current appearance: {resolvedTheme}</span>
        </section>

        <section className="settings-group">
          <h3>File Size Limit</h3>
          <p>Maximum file size (in MB) that AI agents can read.</p>
          <div className="range-row">
            <input
              type="range"
              min="10"
              max="50"
              value={maxFileSizeMb}
              onChange={(event) => setMaxFileSizeMb(Number(event.target.value))}
              aria-label="Maximum file size in MB"
              disabled={actionsDisabled}
            />
            <span>{maxFileSizeMb} MB</span>
          </div>
        </section>

        <section className="settings-group">
          <h3>Browser SSE Mode</h3>
          <p>Allow browser-based AI tools to connect securely to your local files.</p>
          <div className="panel" style={{ marginTop: '0.75rem' }}>
            <div className="inline-row" style={{ justifyContent: 'space-between', marginBottom: '1rem' }}>
              <label htmlFor="sse-toggle" style={{ fontWeight: 500 }}>Enable SSE Server</label>
              <button
                id="sse-toggle"
                type="button"
                className={`button ${sseRunning ? 'button--danger' : 'button--primary'}`}
                onClick={toggleSse}
                disabled={actionsDisabled}
              >
                {sseRunning ? "Stop Server" : "Start Server"}
              </button>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
              <label htmlFor="sse-port" style={{ fontSize: '0.875rem' }}>Port Number</label>
              <input
                id="sse-port"
                type="number"
                className="input-field"
                value={ssePort}
                onChange={(e) => setSsePort(Number(e.target.value))}
                disabled={sseRunning || actionsDisabled}
              />
            </div>

            {sseUrl && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '1rem' }}>
                <label style={{ fontSize: '0.875rem' }}>Connection URL (Paste in MCP config)</label>
                <div className="inline-row">
                  <input
                    type="text"
                    readOnly
                    className="input-field"
                    value={sseUrl}
                    style={{ flex: 1, fontFamily: 'monospace' }}
                  />
                  <button
                    type="button"
                    className="button button--secondary"
                    onClick={() => navigator.clipboard.writeText(sseUrl)}
                  >
                    Copy
                  </button>
                </div>
              </div>
            )}
          </div>
        </section>

        {approvedOrigins.length > 0 && (
          <section className="settings-group">
            <h3>Approved Browser Tools</h3>
            <p>Origins that have permission to connect via SSE.</p>
            <div className="list-stack" style={{ marginTop: '0.5rem' }}>
              {approvedOrigins.map(origin => (
                <div key={origin} className="panel panel--subtle inline-row" style={{ justifyContent: 'space-between', padding: '8px 12px', border: '1px solid var(--border)', borderRadius: '8px' }}>
                  <span style={{ fontSize: '13px', fontFamily: 'monospace' }}>{origin}</span>
                  <button
                    type="button"
                    className="button button--secondary"
                    style={{ minHeight: '26px', padding: '0 8px' }}
                    onClick={() => handleRevoke(origin)}
                  >
                    Revoke
                  </button>
                </div>
              ))}
            </div>
          </section>
        )}

        <section className="settings-group">
          <h3>About OmniDrive</h3>
          <p>Version 0.1.0</p>
          <p>Securely share local files with AI agents via the Model Context Protocol.</p>
        </section>
      </div>
    </section>
  );
}

function SkeletonPanel() {
  return (
    <section className="content-section skeleton-panel" aria-label="Loading content">
      <div className="skeleton-line short" />
      <div className="skeleton-line medium" />
      <div className="skeleton-line long" />
      <div className="skeleton-line medium" />
    </section>
  );
}

export default App;
