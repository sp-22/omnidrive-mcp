This is the unified **Enagager Design Style Guide**. This version removes abstract token names and focuses on direct hex values, layout rules, and the "Frameless" interface philosophy.

---

## 1. Design Philosophy: The "Frameless" Stage

The core of Enagager is the **Unified Workspace**. By removing the traditional top header bar, the sidebar and the main content area occupy the full height of the window. This eliminates visual "shelving" and keeps the user focused on the code and the conversation.

* **Merged Sidebar:** The navigation is a vertical pillar that meets the top of the screen.
* **Headerless Stage:** Functional titles (like "New thread") and action buttons (like "Commit") float within the workspace rather than sitting on a dedicated bar.
* **Minimalist Density:** High information density in the sidebar (13px text) contrasted with a breathable, centered workspace (15px-32px text).

---

## 2. Color Palette

### **Dark Theme**

| Element | Hex Code | Usage |
| --- | --- | --- |
| **Main Stage** | `#0F0F0F` | Primary background for the workspace. |
| **Sidebar & Input** | `#1A1A1A` | Navigation background and the "Ask Codex" container. |
| **Elevated Surfaces** | `#262626` | Hover states for list items and suggested cards. |
| **Dividers/Borders** | `#2E2E2E` | 1px vertical line separating sidebar from stage. |
| **Primary Text** | `#FFFFFF` | Main headings, active threads, and code input. |
| **Muted Text** | `#8E8E8E` | Folder names, timestamps, and placeholder text. |
| **Action Accent** | `#3B82F6` | "Update" pill, active buttons, and cursor focus. |

### **Light Theme**

| Element | Hex Code | Usage |
| --- | --- | --- |
| **Main Stage** | `#F9F9F9` | Primary background for the workspace. |
| **Sidebar & Input** | `#F0F0F0` | Navigation background and input container. |
| **Elevated Surfaces** | `#FFFFFF` | Cards and active hover states. |
| **Dividers/Borders** | `#E5E5E5` | 1px vertical line and subtle card outlines. |
| **Primary Text** | `#1A1A1A` | All high-hierarchy text and content. |
| **Muted Text** | `#737373` | Secondary metadata and folder labels. |
| **Action Accent** | `#2563EB` | Notification pills and primary call-to-actions. |

---

## 3. Sidebar Design & Behavior

The sidebar is a persistent utility anchor with a 240px fixed width.

* **Integrated Top Actions:** "New thread," "Automations," and "Skills" are grouped at the very top left. No padding exists above them to emphasize the "No Header" look.
* **The "Update" Pill:** A bright `#3B82F6` (Dark) or `#2563EB` (Light) pill sits at the top right of the sidebar, serving as the only high-visibility element.
* **Thread Grouping:** * **Folders:** Categorized by project (e.g., *Enagager*). Folder names use muted text.
* **List Items:** 13px font size. Active items receive a `#262626` background.
* **Git Metadata:** Code diffs (e.g., `+117 -97`) are right-aligned in a monospaced font to sit flush against the divider.



---

## 4. Layout & Spacing

A strict **8px grid** maintains alignment across the merged interface.

* **Sidebar Width:** 240px.
* **Main Stage Gutter:** 48px top/bottom padding for the hero section ("Let's build...").
* **Input Area:** Centered at the bottom of the stage with a max-width of 800px.
* **Spacing Units:**
* **4px:** Icon to text.
* **12px:** Indentation for nested thread items.
* **16px:** General padding inside the sidebar and cards.



---

## 5. Copy & Style Guide

The Enagager voice is professional, terse, and mimics developer workflows.

* **Action-Oriented Titles:** Use imperative verbs (e.g., *"Update Gemini model call"* instead of *"The model call was updated"*).
* **Lowercase Accents:** Keep technical terms or file paths in lowercase to feel integrated with code environments.
* **Command Hints:** Placeholder text should always include keyboard or command syntax (e.g., *"Ask Codex anything, @ to add files, / for commands"*).

---
