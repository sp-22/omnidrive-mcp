import { useState, useEffect, useCallback } from "react";
import { getActivityLog, clearActivityLog } from "../lib/tauri";
import type { ActivityEntry, ActivityCategory } from "../lib/types";

export function useActivityLog(initialCategory: ActivityCategory = "all", limit = 100) {
    const [entries, setEntries] = useState<ActivityEntry[]>([]);
    const [activeCategory, setActiveCategory] = useState<ActivityCategory>(initialCategory);
    const [loading, setLoading] = useState(true);

    const fetchLogs = useCallback(async () => {
        try {
            const catArg = activeCategory === "all" ? undefined : activeCategory;
            const data = await getActivityLog(limit, 0, catArg);
            setEntries(data);
        } catch (err) {
            console.error("Failed to fetch activity logs:", err);
        } finally {
            setLoading(false);
        }
    }, [activeCategory, limit]);

    // Initial fetch and polling
    useEffect(() => {
        fetchLogs();
        const interval = setInterval(fetchLogs, 3000); // 3 second polling
        return () => clearInterval(interval);
    }, [fetchLogs]);

    const clear = async () => {
        setLoading(true);
        try {
            await clearActivityLog();
            setEntries([]);
        } catch (err) {
            console.error("Failed to clear logs:", err);
        } finally {
            setLoading(false);
        }
    };

    return {
        entries,
        activeCategory,
        setCategory: setActiveCategory,
        loading,
        clear
    };
}
