//! Dark Mode Color Layer (Obsidian Deep Matte) matching babydra-core

pub const DARK_CSS: &str = r#"
/* ── Text & Base Colors ──────────────────────────────────────────── */
.panel-box,
.topbar-box {
    color: rgba(255, 255, 255, 0.95);
}

.left-pill,
.status-indicators-box,
.sys-monitor-box,
.panel-tray-box,
.workspace-box,
.pill-container,
.panel-notch.active-music {
    background-color: rgb(14, 14, 18);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: rgba(255, 255, 255, 0.95);
}

.left-pill:hover,
.status-indicators-box:hover,
.sys-monitor-box:hover,
.panel-tray-box:hover,
.workspace-box:hover,
.pill-container:hover,
.panel-notch.active-music:not(.popover-open):hover {
    background-color: rgb(24, 24, 28);
    border-color: rgba(255, 255, 255, 0.16);
}

.capsule-separator {
    color: rgba(255, 255, 255, 0.14);
}

.app-title,
.panel-clock,
button.panel-logo-btn,
button.panel-status-btn,
button.panel-clock-btn,
button.panel-tray-item-btn {
    color: #ffffff;
}

button.panel-logo-btn:hover,
button.panel-status-btn:hover,
button.panel-clock-btn:hover,
button.panel-tray-item-btn:hover,
button.app-title-btn:hover,
button.func-btn:hover,
.fluent-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
    color: #ffffff;
}

button.panel-logo-btn:active,
button.panel-status-btn:active,
button.panel-clock-btn:active,
button.panel-tray-item-btn:active,
button.app-title-btn:active,
button.func-btn:active,
.fluent-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
    color: #ffffff;
}

button.app-title-btn,
button.func-btn,
.fluent-btn {
    color: rgba(255, 255, 255, 0.90);
}

.status-text,
.status-icon {
    color: rgba(255, 255, 255, 0.90);
}

/* ── Dynamic Island ──────────────────────────────────────────────── */
.island-bracket {
    color: rgba(255, 255, 255, 0.40);
}

.island-wrapper:hover .island-bracket,
.panel-notch:hover .island-bracket {
    color: #60a5fa;
}

.notch-player-text {
    color: #ffffff;
}

.notch-system-value {
    color: #60a5fa;
}

.visualizer-bar {
    background-color: #3b82f6;
}

.media-control-btn {
    color: #ffffff;
}

.media-control-btn:hover {
    background-color: rgba(255, 255, 255, 0.12);
}

/* ── Popovers & Tooltips ─────────────────────────────────────────── */
popover>contents,
popover.menu>contents,
popover.status-popover>contents,
popover.tooltip-popover>contents {
    background-color: rgb(13, 13, 17);
    border: 1px solid rgba(255, 255, 255, 0.10);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.65);
    color: #ffffff;
}

.status-popover-header,
.tooltip-popover-header {
    color: #ffffff;
}

.status-popover-key,
.tooltip-popover-key {
    color: rgba(255, 255, 255, 0.50);
}

.status-popover-val,
.tooltip-popover-val {
    color: #ffffff;
}

.tile-title {
    color: #ffffff;
}

.tile-subtitle {
    color: rgba(255, 255, 255, 0.75);
}

.control-square-label {
    color: rgba(255, 255, 255, 0.60);
}

popover.sys-monitor-popover progressbar trough {
    background-color: rgba(255, 255, 255, 0.08);
}

popover.sys-monitor-popover progressbar progress {
    background-color: #3b82f6;
}

popover.sys-monitor-popover .sys-chart {
    background-color: rgba(255, 255, 255, 0.03);
}

/* ── Context Menu Overlay ────────────────────────────────────────── */
.context-menu-box {
    background-color: rgb(14, 14, 18);
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.65);
    color: rgba(255, 255, 255, 0.90);
}

button.context-menu-item {
    color: rgba(255, 255, 255, 0.90);
}

button.context-menu-item label {
    color: rgba(255, 255, 255, 0.90);
}

button.context-menu-item:focus,
button.context-menu-item:hover {
    background-color: rgba(255, 255, 255, 0.08);
    color: #ffffff;
}

button.context-menu-item:focus label,
button.context-menu-item:hover label {
    color: #ffffff;
}

button.context-menu-item.destructive:hover {
    background-color: rgba(239, 68, 68, 0.15);
}

.shortcut-label {
    color: rgba(255, 255, 255, 0.50);
}

.context-menu-separator,
.menu-sep {
    background-color: rgba(255, 255, 255, 0.10);
}

button.context-menu-footer-btn {
    color: rgba(255, 255, 255, 0.85);
}

button.context-menu-footer-btn:hover,
button.context-menu-footer-btn:active {
    background-color: rgba(255, 255, 255, 0.08);
    color: #ffffff;
}

/* ── Desktop Shell ───────────────────────────────────────────────── */
.desktop-wallpaper-container {
    background-color: #0b0c10;
}

.desktop-icon:hover {
    background-color: rgba(255, 255, 255, 0.10);
    border: 1px solid rgba(255, 255, 255, 0.16);
}

.desktop-icon.selected {
    background-color: rgba(255, 255, 255, 0.20);
    border: 1px solid rgba(255, 255, 255, 0.35);
}

.desktop-icon-label {
    color: #ffffff;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.95), 0 0 2px rgba(0, 0, 0, 0.8);
}

.desktop-rubberband {
    background-color: rgba(59, 130, 246, 0.20);
    border: 1px solid rgba(59, 130, 246, 0.60);
}

/* ── Standard UI Kit Components ──────────────────────────────────── */
button.baby-button {
    border: 1px solid rgba(255, 255, 255, 0.12);
    background-color: rgba(255, 255, 255, 0.06);
    color: #ffffff;
}

button.baby-button:hover {
    background-color: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.20);
    color: #ffffff;
}

button.btn-primary {
    background-color: #3b82f6;
    color: #ffffff;
}

button.btn-primary:hover {
    background-color: #2563eb;
}

.circular-disc {
    background-color: rgb(24, 25, 31);
    border: 1px solid rgba(255, 255, 255, 0.14);
    color: #3b82f6;
}

.badge {
    background: rgba(59, 130, 246, 0.18);
    color: #60a5fa;
}

.badge-subtle {
    background: rgba(255, 255, 255, 0.10);
    color: rgba(255, 255, 255, 0.85);
}

scrollbar slider {
    background-color: rgba(255, 255, 255, 0.20);
}

scrollbar slider:hover,
scrollbar slider:hover:active {
    background-color: rgba(255, 255, 255, 0.35);
}

box.screenshot-toolbar {
    background-color: rgba(20, 20, 26, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.50);
}

button.screenshot-toolbar-btn {
    color: #ffffff;
}

button.screenshot-toolbar-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
}

label.capsule-separator {
    color: rgba(255, 255, 255, 0.35);
}

button.color-dot-btn.color-dot-white {
    background-color: #ffffff;
}
"#;

