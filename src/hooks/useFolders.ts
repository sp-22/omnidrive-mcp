import { useState, useEffect, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { SharedFolder, FolderScanResult } from "../lib/types";
import * as api from "../lib/tauri";

export function useFolders() {
    const [folders, setFolders] = useState<SharedFolder[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [scanResult, setScanResult] = useState<FolderScanResult | null>(null);
    const [showScanWarning, setShowScanWarning] = useState(false);

    const refresh = useCallback(async () => {
        try {
            const result = await api.listFolders();
            setFolders(result);
            setError(null);
        } catch (e) {
            setError(String(e));
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        refresh();
    }, [refresh]);

    const addFolder = useCallback(async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select a folder to share with AI agents",
            });

            if (!selected) return;

            const path = typeof selected === "string" ? selected : selected;
            const scan = await api.addFolder(path as string);
            setScanResult(scan);

            if (scan.unsupported_files > 0) {
                setShowScanWarning(true);
            }

            await refresh();
            setError(null);
        } catch (e) {
            const msg = String(e);
            if (msg.startsWith("OVERLAP:")) {
                setError(msg.replace("OVERLAP:", ""));
            } else {
                setError(msg);
            }
        }
    }, [refresh]);

    const removeFolder = useCallback(async (path: string) => {
        try {
            await api.removeFolder(path);
            await refresh();
            setError(null);
        } catch (e) {
            setError(String(e));
        }
    }, [refresh]);

    const setPermission = useCallback(async (path: string, permission: string) => {
        try {
            await api.togglePermission(path, permission as any);
            await refresh();
        } catch (e) {
            setError(String(e));
        }
    }, [refresh]);

    const toggleEnabled = useCallback(async (path: string, enabled: boolean) => {
        try {
            await api.toggleFolderEnabled(path, enabled);
            await refresh();
        } catch (e) {
            setError(String(e));
        }
    }, [refresh]);

    const dismissScanWarning = useCallback(() => {
        setShowScanWarning(false);
        setScanResult(null);
    }, []);

    return {
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
        refresh,
        setError,
    };
}
