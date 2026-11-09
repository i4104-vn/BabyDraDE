//! Light Mode Color Layer matching babydra-core

pub const LIGHT_CSS: &str = r#"
/* ── Text & Base Colors ──────────────────────────────────────────── */
.panel-box,
.topbar-box {
    color: rgba(17, 24, 39, 0.95);
}

.left-pill,
.status-indicators-box,
.sys-monitor-box,
.panel-tray-box,
.workspace-box,
.pill-container,
.panel-notch.active-music {
    background-color: #ffffff;
    border: 1px solid rgba(0, 0, 0, 0.08);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
    color: #111827;
}

.left-pill:hover,
.status-indicators-box:hover,
.sys-monitor-box:hover,
.panel-tray-box:hover,
.workspace-box:hover,
.pill-container:hover,
.panel-notch.active-music:not(.popover-open):hover {
    background-color: #f3f4f6;
    border-color: rgba(0, 0, 0, 0.14);
}

.capsule-separator {
    color: rgba(0, 0, 0, 0.12);
}

.app-title,
.panel-clock,
button.panel-logo-btn,
button.panel-status-btn,
button.panel-clock-btn,
button.panel-tray-item-btn {
    color: #111827;
}

button.panel-logo-btn:hover,
button.panel-status-btn:hover,
button.panel-clock-btn:hover,
button.panel-tray-item-btn:hover,
button.app-title-btn:hover,
button.func-btn:hover,
.fluent-btn:hover {
    background-color: rgba(0, 0, 0, 0.06);
    color: #111827;
}

button.panel-logo-btn:active,
button.panel-status-btn:active,
button.panel-clock-btn:active,
button.panel-tray-item-btn:active,
button.app-title-btn:active,
button.func-btn:active,
.fluent-btn:active {
    background-color: rgba(0, 0, 0, 0.12);
    color: #111827;
}

button.app-title-btn,
button.func-btn,
.fluent-btn {
    color: rgba(17, 24, 39, 0.85);
}

.status-text,
.status-icon {
    color: rgba(17, 24, 39, 0.85);
}

/* ── Dynamic Island ──────────────────────────────────────────────── */
.island-bracket {
    color: rgba(0, 0, 0, 0.35);
}

.island-wrapper:hover .island-bracket,
.panel-notch:hover .island-bracket {
    color: #2563eb;
}

.notch-player-text {
    color: #111827;
}

.notch-system-value {
    color: #2563eb;
}

.visualizer-bar {
    background-color: #2563eb;
}

.media-control-btn {
    color: #111827;
}

.media-control-btn:hover {
    background-color: rgba(0, 0, 0, 0.08);
}

/* ── Popovers & Tooltips ─────────────────────────────────────────── */
popover>contents,
popover.menu>contents,
popover.status-popover>contents,
popover.tooltip-popover>contents {
    background-color: #ffffff;
    border: 1px solid rgba(0, 0, 0, 0.10);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.12);
    color: #111827;
}

.status-popover-header,
.tooltip-popover-header {
    color: #111827;
}

.status-popover-key,
.tooltip-popover-key {
    color: rgba(0, 0, 0, 0.55);
}

.status-popover-val,
.tooltip-popover-val {
    color: #111827;
}

.tile-title {
    color: #111827;
}

.tile-subtitle {
    color: rgba(0, 0, 0, 0.65);
}

.control-square-label {
    color: rgba(0, 0, 0, 0.50);
}

popover.sys-monitor-popover progressbar trough {
    background-color: rgba(0, 0, 0, 0.08);
}

popover.sys-monitor-popover progressbar progress {
    background-color: #2563eb;
}

popover.sys-monitor-popover .sys-chart {
    background-color: rgba(0, 0, 0, 0.03);
}

/* ── Context Menu Overlay ────────────────────────────────────────── */
.context-menu-box {
    background-color: #ffffff;
    border: 1px solid rgba(0, 0, 0, 0.10);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.14);
    color: #111827;
}

button.context-menu-item {
    color: #111827;
}

button.context-menu-item label {
    color: #111827;
}

button.context-menu-item:focus,
button.context-menu-item:hover {
    background-color: rgba(0, 0, 0, 0.05);
    color: #111827;
}

button.context-menu-item:focus label,
button.context-menu-item:hover label {
    color: #111827;
}

button.context-menu-item.destructive:hover {
    background-color: rgba(239, 68, 68, 0.12);
}

.shortcut-label {
    color: rgba(0, 0, 0, 0.45);
}

.context-menu-separator,
.menu-sep {
    background-color: rgba(0, 0, 0, 0.08);
}

button.context-menu-footer-btn {
    color: rgba(0, 0, 0, 0.70);
}

button.context-menu-footer-btn:hover,
button.context-menu-footer-btn:active {
    background-color: rgba(0, 0, 0, 0.06);
    color: #111827;
}

/* ── Desktop Shell ───────────────────────────────────────────────── */
.desktop-wallpaper-container {
    background-color: #f3f4f6;
}

.desktop-icon:hover {
    background-color: rgba(255, 255, 255, 0.16);
    border: 1px solid rgba(255, 255, 255, 0.25);
}

.desktop-icon.selected {
    background-color: rgba(255, 255, 255, 0.30);
    border: 1px solid rgba(255, 255, 255, 0.50);
}

.desktop-icon-label {
    color: #ffffff;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.95), 0 0 2px rgba(0, 0, 0, 0.8);
}

.desktop-rubberband {
    background-color: rgba(37, 99, 235, 0.20);
    border: 1px solid rgba(37, 99, 235, 0.60);
}

/* ── Standard UI Kit Components ──────────────────────────────────── */
button.baby-button {
    border: 1px solid rgba(0, 0, 0, 0.10);
    background-color: rgba(0, 0, 0, 0.04);
    color: #111827;
}

button.baby-button:hover {
    background-color: rgba(0, 0, 0, 0.08);
    border-color: rgba(0, 0, 0, 0.16);
    color: #111827;
}

button.btn-primary {
    background-color: #2563eb;
    color: #ffffff;
}

button.btn-primary:hover {
    background-color: #1d4ed8;
}

.circular-disc {
    background-color: #f3f4f6;
    border: 1px solid rgba(0, 0, 0, 0.12);
    color: #2563eb;
}

.badge {
    background: rgba(37, 99, 235, 0.12);
    color: #2563eb;
}

.badge-subtle {
    background: rgba(0, 0, 0, 0.06);
    color: rgba(0, 0, 0, 0.75);
}

scrollbar slider {
    background-color: rgba(0, 0, 0, 0.20);
}

scrollbar slider:hover,
scrollbar slider:hover:active {
    background-color: rgba(0, 0, 0, 0.35);
}
"#;

