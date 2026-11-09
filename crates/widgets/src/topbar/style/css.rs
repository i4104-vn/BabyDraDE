//! TopBar-specific CSS styling matching babydra-panel and babydra-core 100%

pub const TOPBAR_CSS: &str = r#"
/* ── Window & Topbar Container ───────────────────────────────────── */
window.panel-window {
    background-color: transparent;
    background-image: none;
    box-shadow: none;
    border: none;
}

.panel-box,
.topbar-box {
    background-color: transparent;
    background-image: none;
    margin: 0;
    padding: 0px 4px;
    min-height: 36px;
    border: none;
    box-shadow: none;
    color: rgba(255, 255, 255, 0.95);
}

/* ── Universal Reset for Topbar Buttons ──────────────────────────── */
.panel-box button,
.topbar-box button {
    background-color: transparent;
    background-image: none;
    border: none;
    outline: none;
    box-shadow: none;
    margin: 0;
    transition: background-color 150ms ease, color 150ms ease;
}

.panel-box button:hover,
.panel-box button:active,
.panel-box button:focus,
.topbar-box button:hover,
.topbar-box button:active,
.topbar-box button:focus {
    background-image: none;
    border: none;
    outline: none;
    box-shadow: none;
}

/* ── Left Pill (Logo & App Menu) ─────────────────────────────────── */
.left-pill,
.workspace-box {
    margin-left: 4px;
    margin-right: 4px;
    background-color: rgb(14, 14, 18);
    background-image: none;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    padding: 2px 8px;
    min-height: 26px;
    box-shadow: none;
}

button.panel-logo-btn {
    background-color: transparent;
    background-image: none;
    border: none;
    border-radius: 6px;
    padding: 2px 4px;
    margin: 0;
    box-shadow: none;
    min-height: 20px;
    min-width: 20px;
    color: #ffffff;
    transition: background-color 150ms ease;
}

button.panel-logo-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
    border: none;
    box-shadow: none;
}

button.panel-logo-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
    border: none;
    box-shadow: none;
}

.capsule-separator {
    font-size: 9px;
    margin: 0 4px;
    color: rgba(255, 255, 255, 0.14);
}

.app-title {
    font-weight: 650;
    color: #ffffff;
    font-size: 13px;
    margin-right: 4px;
}

button.app-title-btn,
button.func-btn {
    background-color: transparent;
    background-image: none;
    border: none;
    border-radius: 6px;
    padding: 2px 6px;
    margin: 0;
    box-shadow: none;
    min-height: 20px;
    color: rgba(255, 255, 255, 0.90);
    font-size: 12.5px;
    font-weight: 500;
    transition: background-color 150ms ease, color 150ms ease;
}

button.app-title-btn:hover,
button.func-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
    border: none;
    box-shadow: none;
    color: #ffffff;
}

button.app-title-btn:active,
button.func-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
    border: none;
    box-shadow: none;
}

/* ── Dynamic Island (Center Notch) ───────────────────────────────── */
.island-wrapper {
    background-color: transparent;
    background-image: none;
    border: none;
    box-shadow: none;
}

.panel-notch {
    background-color: transparent;
    border: none;
    min-width: 0;
    min-height: 0;
    margin: 0;
    padding: 0;
    font-size: 12px;
    box-shadow: none;
    transition: all 400ms cubic-bezier(0.16, 1, 0.3, 1);
}

.panel-notch.active-music {
    border-radius: 9999px;
    min-width: 0;
    min-height: 26px;
    padding: 0 10px;
    margin-top: 0;
    background-color: rgb(14, 14, 18);
    opacity: 1.0;
    border: 1px solid rgba(255, 255, 255, 0.12);
}

.panel-notch.active-music:not(.popover-open):hover {
    background-color: rgb(24, 24, 28);
    border-color: rgba(255, 255, 255, 0.16);
}

.island-bracket {
    font-size: 12px;
    font-weight: 800;
    line-height: 1;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace, sans-serif;
    color: rgba(255, 255, 255, 0.40);
    padding: 0;
    margin: 0;
    min-width: 8px;
    min-height: 16px;
    transition: all 180ms cubic-bezier(0.16, 1, 0.3, 1);
}

.island-bracket.left {
    margin-right: 2px;
    margin-left: -2px;
}

.island-bracket.right {
    margin-left: 2px;
    margin-right: -2px;
}

.island-wrapper:hover .island-bracket,
.panel-notch:hover .island-bracket {
    color: #60a5fa;
    text-shadow: none;
}

.notch-content {
    background-color: transparent;
    margin: 0;
    padding: 0;
}

.notch-player-text {
    font-size: 12px;
    font-weight: 600;
    color: #ffffff;
    margin: 0 6px;
}

.notch-system-value {
    font-size: 11px;
    font-weight: 600;
    color: #60a5fa;
    font-feature-settings: "tnum";
}

/* ── Right Section ───────────────────────────────────────────────── */
/* Tray */
.panel-tray-box {
    margin-right: 4px;
    background-color: rgb(14, 14, 18);
    background-image: none;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    padding: 2px 6px;
    min-height: 26px;
    box-shadow: none;
}

button.panel-tray-item-btn {
    background-color: transparent;
    background-image: none;
    border: none;
    padding: 2px 4px;
    margin: 0;
    border-radius: 6px;
    box-shadow: none;
    min-height: 20px;
    min-width: 20px;
    transition: all 150ms ease;
    color: #ffffff;
}

button.panel-tray-item-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
    border: none;
    box-shadow: none;
}

button.panel-tray-item-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
    border: none;
    box-shadow: none;
}

/* System Monitor */
.sys-monitor-box {
    background-color: rgb(14, 14, 18);
    background-image: none;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    padding: 2px 10px;
    min-height: 26px;
    box-shadow: none;
    transition: all 200ms ease;
}

.sys-monitor-box:hover {
    background-color: rgb(24, 24, 28);
    border-color: rgba(255, 255, 255, 0.16);
}

/* Unified Status Indicators Capsule (Status + Clock) */
.status-indicators-box {
    background-color: rgb(14, 14, 18);
    background-image: none;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    padding: 2px 8px;
    min-height: 26px;
    box-shadow: none;
}

.status-text {
    font-size: 12px;
    font-weight: 600;
    text-shadow: none;
    margin-left: 2px;
    margin-right: 2px;
    color: rgba(255, 255, 255, 0.90);
    font-feature-settings: "tnum";
}

.status-icon {
    text-shadow: none;
    margin: 0 4px;
    color: rgba(255, 255, 255, 0.90);
}

button.panel-status-btn {
    background-color: transparent;
    background-image: none;
    border: none;
    padding: 2px 6px;
    margin: 0;
    box-shadow: none;
    min-height: 20px;
    border-radius: 9999px;
    color: #ffffff;
    transition: all 150ms ease;
}

button.panel-status-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
    border: none;
    box-shadow: none;
}

button.panel-status-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
    border: none;
    box-shadow: none;
}

.panel-clock {
    font-weight: 600;
    font-size: 12px;
    text-shadow: none;
    color: #ffffff;
    font-feature-settings: "tnum";
}

button.panel-clock-btn {
    background-color: transparent;
    background-image: none;
    border: none;
    padding: 2px 10px;
    margin: 0;
    box-shadow: none;
    min-height: 20px;
    border-radius: 9999px;
    transition: all 150ms ease;
    color: #ffffff;
}

button.panel-clock-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
    border: none;
    box-shadow: none;
}

button.panel-clock-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
    border: none;
    box-shadow: none;
}

.clock-bell-icon {
    opacity: 0.85;
}

.clock-bell-dot {
    background-color: #f43f5e;
    background-image: none;
    border: none;
    box-shadow: none;
    border-radius: 9999px;
    min-width: 7px;
    min-height: 7px;
    margin-top: -2px;
    margin-right: -2px;
}

.clock-bell-box {
    border-radius: 6px;
    padding: 2px 4px;
    margin: 0;
    transition: background-color 150ms ease;
}

.clock-bell-box:hover {
    background-color: rgba(255, 255, 255, 0.12);
}

/* ── Popovers & Cards ────────────────────────────────────────────── */
popover {
    background-color: transparent;
    padding: 0;
    margin: 0;
    box-shadow: none;
    border: none;
}

popover>contents {
    background-color: rgb(14, 14, 18);
    opacity: 1.0;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 14px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.50);
    padding: 10px 14px;
}

popover>arrow {
    background: none;
    border: none;
}

.status-popover-card {
    min-width: 200px;
}

.status-popover-header {
    font-weight: 700;
    font-size: 14px;
    margin-bottom: 4px;
    color: rgba(255, 255, 255, 0.95);
}

.status-popover-sep {
    opacity: 0.15;
    margin-bottom: 6px;
}

.status-popover-key {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.60);
}

.status-popover-val {
    font-size: 12px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.95);
}

.status-popover-val.speed-low {
    color: #ef4444;
    font-weight: 700;
}

.status-popover-val.speed-medium {
    color: #f59e0b;
    font-weight: 700;
}

.status-popover-val.speed-high {
    color: #22c55e;
    font-weight: 700;
}

popover.sys-monitor-popover>contents {
    padding: 14px;
}

popover.sys-monitor-popover progressbar trough {
    min-height: 6px;
    border-radius: 9999px;
    border: none;
    background-color: rgba(255, 255, 255, 0.08);
}

popover.sys-monitor-popover progressbar progress {
    border-radius: 9999px;
    background-color: #3b82f6;
}

popover.sys-monitor-popover .sys-chart {
    border: none;
    border-radius: 10px;
    margin-top: 6px;
    margin-bottom: 10px;
    background-color: rgba(255, 255, 255, 0.03);
}

.tile-title {
    font-size: 13.5px;
    font-weight: 650;
    color: #FFFFFF;
}

.tile-subtitle {
    font-size: 12px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.75);
}

.control-square-label {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.60);
    font-feature-settings: "tnum";
}
"#;
