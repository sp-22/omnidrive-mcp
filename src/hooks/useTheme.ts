import { useEffect, useMemo, useState } from "react";
import type { ResolvedTheme, ThemePreference } from "../lib/types";

const STORAGE_KEY = "omnidrive.theme-preference";
const COLOR_SCHEME_QUERY = "(prefers-color-scheme: dark)";

function isValidPreference(value: string | null): value is ThemePreference {
    return value === "system" || value === "light" || value === "dark";
}

function readStoredPreference(): ThemePreference {
    try {
        const stored = window.localStorage.getItem(STORAGE_KEY);
        if (isValidPreference(stored)) {
            return stored;
        }
    } catch {
        // Ignore storage errors and fall back to system.
    }

    return "system";
}

function resolveSystemTheme(): ResolvedTheme {
    return window.matchMedia(COLOR_SCHEME_QUERY).matches ? "dark" : "light";
}

export function useTheme() {
    const [preference, setPreference] = useState<ThemePreference>(readStoredPreference);
    const [systemTheme, setSystemTheme] = useState<ResolvedTheme>(resolveSystemTheme);

    const resolvedTheme = useMemo<ResolvedTheme>(() => {
        return preference === "system" ? systemTheme : preference;
    }, [preference, systemTheme]);

    useEffect(() => {
        const mediaQuery = window.matchMedia(COLOR_SCHEME_QUERY);

        const onChange = (event: MediaQueryListEvent) => {
            setSystemTheme(event.matches ? "dark" : "light");
        };

        setSystemTheme(mediaQuery.matches ? "dark" : "light");
        mediaQuery.addEventListener("change", onChange);

        return () => {
            mediaQuery.removeEventListener("change", onChange);
        };
    }, []);

    useEffect(() => {
        try {
            window.localStorage.setItem(STORAGE_KEY, preference);
        } catch {
            // Ignore storage write errors.
        }
    }, [preference]);

    useEffect(() => {
        const root = document.documentElement;
        root.dataset.theme = resolvedTheme;
        root.style.colorScheme = resolvedTheme;
    }, [resolvedTheme]);

    return {
        preference,
        resolvedTheme,
        setPreference,
    };
}
