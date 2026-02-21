import type { ReactNode } from "react";

interface AppShellProps {
    sidebar: ReactNode;
    children: ReactNode;
}

export function AppShell({ sidebar, children }: AppShellProps) {
    return (
        <div className="app-root">
            <div className="app-window">
                <div className="app-body">
                    <aside className="app-sidebar">{sidebar}</aside>
                    <main className="app-main">{children}</main>
                </div>
            </div>
        </div>
    );
}
