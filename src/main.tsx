import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

const THEME_STORAGE_KEY = "omnidrive.theme-preference";

function applyInitialTheme() {
  let resolved: "light" | "dark" = "light";

  try {
    const preference = window.localStorage.getItem(THEME_STORAGE_KEY);
    if (preference === "light" || preference === "dark") {
      resolved = preference;
    } else {
      resolved = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }
  } catch {
    resolved = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }

  document.documentElement.dataset.theme = resolved;
  document.documentElement.style.colorScheme = resolved;
}

applyInitialTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
