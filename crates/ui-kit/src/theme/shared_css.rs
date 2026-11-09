//! Shared structural stylesheet — layout, spacing, typography, component structure.
//! Color values are handled by the dark and light theme layers.

pub const SHARED_CSS: &str = r#"
/* ── Global Font & Icon Reset ────────────────────────────────────────── */
* {
    font-family: 'Quicksand', 'Segoe UI Variable Static Text', 'Segoe UI', 'Inter', system-ui, -apple-system, sans-serif;
    -gtk-icon-style: symbolic;
}

/* Tabular figures for metrics and clocks */
.panel-clock,
.status-popover-val,
.tooltip-popover-val,
.status-text,
.sys-monitor-val,
.notch-system-value,
.control-square-label,
.shortcut-label {
    font-feature-settings: "tnum";
    font-variant-numeric: tabular-nums;
}

/* Transparent Window Base */
window,
window.background,
window.panel-window,
window.desktop-window {
    background: transparent;
    background-color: transparent;
    box-shadow: none;
    border: none;
}

/* Universal Button Reset */
button {
    font-family: 'Quicksand', 'Segoe UI Variable Static Text', 'Segoe UI', 'Inter', system-ui, -apple-system, sans-serif;
    background-color: transparent;
    background-image: none;
    border: none;
    outline: none;
    box-shadow: none;
    margin: 0;
    padding: 0;
    min-height: 0;
    min-width: 0;
    transition: background-color 150ms ease, color 150ms ease;
}

button:hover,
button:active,
button:focus,
button:checked {
    background-image: none;
    border: none;
    outline: none;
    box-shadow: none;
}

/* ── Topbar Containers & Capsules ────────────────────────────────────── */
.panel-box,
.topbar-box {
    background-color: transparent;
    background-image: none;
    margin: 0;
    padding: 0px 4px;
    min-height: 36px;
    border: none;
    box-shadow: none;
}

.left-pill,
.status-indicators-box,
.sys-monitor-box,
.panel-tray-box,
.workspace-box,
.pill-container {
    background-image: none;
    border-radius: 9999px;
    padding: 2px 8px;
    min-height: 26px;
    box-shadow: none;
    transition: background-color 200ms ease, border-color 200ms ease;
}

.left-pill,
.workspace-box {
    margin-left: 4px;
    margin-right: 4px;
}

.panel-tray-box {
    margin-right: 4px;
    padding: 2px 6px;
}

.sys-monitor-box {
    padding: 2px 10px;
}

/* Vertical Separator */
.capsule-separator,
.separator {
    font-size: 9px;
    margin: 0 4px;
    font-weight: 300;
}

/* Buttons inside Capsules */
button.panel-logo-btn {
    border-radius: 6px;
    padding: 2px 4px;
    margin: 0;
    min-height: 20px;
    min-width: 20px;
    transition: background-color 150ms ease;
}

.app-title {
    font-weight: 650;
    font-size: 13px;
    margin-right: 4px;
}

button.app-title-btn,
button.func-btn,
.fluent-btn {
    border-radius: 6px;
    padding: 2px 6px;
    margin: 0;
    min-height: 20px;
    font-size: 12.5px;
    font-weight: 500;
    transition: background-color 150ms ease, color 150ms ease;
}

button.panel-status-btn {
    padding: 2px 6px;
    margin: 0;
    min-height: 20px;
    border-radius: 9999px;
    transition: all 150ms ease;
}

.status-text {
    font-size: 12px;
    font-weight: 600;
    margin-left: 2px;
    margin-right: 2px;
}

.status-icon {
    margin: 0 4px;
}

.panel-clock {
    font-weight: 600;
    font-size: 12px;
}

button.panel-clock-btn {
    padding: 2px 10px;
    margin: 0;
    min-height: 20px;
    border-radius: 9999px;
    transition: all 150ms ease;
}

button.panel-tray-item-btn {
    padding: 2px 4px;
    margin: 0;
    border-radius: 6px;
    min-height: 20px;
    min-width: 20px;
    transition: all 150ms ease;
}

/* Clock Bell Dot */
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

/* ── Dynamic Island (Center Notch) ───────────────────────────────── */
.island-wrapper {
    background-color: transparent;
    background-image: none;
    border: none;
    box-shadow: none;
}

.panel-notch,
.dynamic-island {
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
}

.island-bracket {
    font-size: 12px;
    font-weight: 800;
    line-height: 1;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace, sans-serif;
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

.notch-content {
    background-color: transparent;
    margin: 0;
    padding: 0;
}

.notch-player-text {
    font-size: 12px;
    font-weight: 600;
    margin: 0 6px;
}

.notch-system-value {
    font-size: 11px;
    font-weight: 600;
}

.notch-visualizer {
    margin-left: 4px;
}

.visualizer-bar {
    border-radius: 9999px;
    margin: 0 1px;
}

.recording-notch {
    color: #ef4444;
}

.recording-dot {
    color: #ef4444;
    font-size: 10px;
    margin-right: 2px;
}

.recording-title {
    color: #ef4444;
    font-weight: 700;
    font-size: 11px;
}

.recording-timer {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.85);
}

.media-control-btn {
    border-radius: 9999px;
    padding: 4px 8px;
    min-height: 28px;
    min-width: 28px;
}

/* ── Popovers & Tooltips ─────────────────────────────────────────── */
popover {
    background-color: transparent;
    padding: 0;
    margin: 0;
    box-shadow: none;
    border: none;
}

popover>arrow {
    display: none;
    background: none;
    border: none;
}

popover>contents,
popover.menu>contents,
popover.status-popover>contents,
popover.tooltip-popover>contents {
    background-image: none;
    opacity: 1.0;
    border-radius: 18px;
    padding: 14px 18px;
}

.status-popover-card,
.tooltip-popover-card {
    min-width: 220px;
}

.status-popover-card.tooltip-popover-simple,
.tooltip-popover-card.tooltip-popover-simple {
    min-width: unset;
    padding: 2px 4px;
}

.status-popover-header,
.tooltip-popover-header {
    font-weight: 700;
    font-size: 14px;
    margin-top: 0;
    margin-bottom: 12px;
}

.status-popover-sep,
.tooltip-popover-sep {
    display: none;
}

.status-popover-row,
.tooltip-popover-row {
    margin-bottom: 6px;
}

.status-popover-key,
.tooltip-popover-key {
    font-size: 12px;
    font-weight: 500;
}

.status-popover-val,
.tooltip-popover-val {
    font-size: 12px;
    font-weight: 700;
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
    padding: 16px 18px;
    border-radius: 18px;
}

popover.sys-monitor-popover progressbar trough {
    min-height: 6px;
    border-radius: 9999px;
    border: none;
}

popover.sys-monitor-popover progressbar progress {
    border-radius: 9999px;
}

popover.sys-monitor-popover .sys-chart {
    border: none;
    border-radius: 10px;
    margin-top: 6px;
    margin-bottom: 10px;
}

.tile-title {
    font-size: 13.5px;
    font-weight: 650;
}

.tile-subtitle {
    font-size: 12px;
    font-weight: 500;
}

.control-square-label {
    font-size: 11px;
}

/* ── Context Menu Overlay ────────────────────────────────────────── */
@keyframes context-menu-pop {
    0% {
        opacity: 0;
        margin-top: -6px;
    }
    100% {
        opacity: 1;
        margin-top: 0px;
    }
}

popover.context-menu-popover,
popover.explore-popover,
popover.tray-context-menu,
popover.desktop-context-menu,
popover.context-menu-popover>contents,
popover.explore-popover>contents,
popover.tray-context-menu>contents,
popover.desktop-context-menu>contents {
    background: none;
    background-color: transparent;
    background-image: none;
    border: none;
    border-radius: 0;
    box-shadow: none;
    padding: 0;
    margin: 0;
}

popover.context-menu-popover>arrow,
popover.explore-popover>arrow,
popover.tray-context-menu>arrow,
popover.desktop-context-menu>arrow {
    display: none;
    background: none;
    border: none;
}

.context-menu-box {
    border-radius: 14px;
    padding: 4px;
    animation: context-menu-pop 160ms cubic-bezier(0.16, 1, 0.3, 1);
}

button.context-menu-item {
    border-radius: 8px;
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 500;
    min-height: 28px;
    transition: background-color 140ms ease, color 140ms ease, padding-left 140ms ease;
}

button.context-menu-item.destructive {
    color: #ef4444;
}

button.context-menu-item.destructive label {
    color: #ef4444;
}

.shortcut-label {
    font-size: 12px;
    margin-left: 12px;
    transition: opacity 140ms ease;
}

.context-menu-separator,
.menu-sep {
    margin-top: 4px;
    margin-bottom: 4px;
    margin-left: 6px;
    margin-right: 6px;
    min-height: 1px;
    opacity: 1;
}

.context-menu-header,
.group-header-label {
    font-weight: 700;
    font-size: 11px;
    letter-spacing: 0.8px;
    text-transform: uppercase;
    color: #60a5fa;
    margin-top: 6px;
    margin-bottom: 2px;
    padding-left: 10px;
    opacity: 0.85;
}

.context-menu-footer {
    margin-top: 4px;
    padding-top: 4px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
}

button.context-menu-footer-btn {
    border-radius: 6px;
    padding: 4px;
    margin: 2px;
    min-width: 26px;
    min-height: 26px;
    transition: background-color 140ms ease, color 140ms ease;
}

/* ── Desktop Shell ───────────────────────────────────────────────── */
.desktop-window {
    background-color: transparent;
    background-image: none;
    border: none;
    box-shadow: none;
}

.desktop-grid {
    background: transparent;
    background-color: transparent;
}

.desktop-icon {
    min-width: 84px;
    min-height: 88px;
    border-radius: 10px;
    padding: 6px 4px 6px 4px;
    background: transparent;
    background-color: transparent;
    border: 1px solid transparent;
    transition: background-color 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
}

.desktop-icon-image {
    margin-bottom: 3px;
}

.desktop-icon-label {
    font-size: 12px;
    font-weight: 500;
    line-height: 1.2;
    padding: 0 2px;
}

/* ── Standard UI Kit Components ──────────────────────────────────── */
button.baby-button {
    border-radius: 10px;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 600;
    background-image: none;
    box-shadow: none;
    transition: background-color 200ms ease, border-color 200ms ease, color 200ms ease;
}

button.btn-primary {
    border-radius: 9999px;
    border: none;
    font-size: 14px;
    font-weight: 600;
    padding: 8px 18px;
    box-shadow: none;
    transition: background-color 150ms ease;
}

.circular-disc {
    border-radius: 9999px;
    padding: 2px 6px;
    font-size: 11px;
    min-width: 20px;
    min-height: 20px;
}

.badge {
    border-radius: 9999px;
    padding: 1px 7px;
    font-weight: 700;
    font-size: 11px;
}

/* ── Scrollbars ──────────────────────────────────────────────────── */
scrollbar {
    background: transparent;
    border: none;
}

scrollbar trough {
    background-color: transparent;
    border: none;
}

scrollbar slider {
    border: 4px solid transparent;
    background-clip: padding-box;
    border-radius: 10px;
    min-width: 6px;
    min-height: 6px;
    transition: background-color 0.2s ease, border-width 0.2s ease;
}

scrollbar slider:hover,
scrollbar slider:hover:active {
    border-width: 2px;
}

scrollbar button {
    min-width: 0;
    min-height: 0;
    opacity: 0;
    padding: 0;
    margin: 0;
    border: none;
    background: none;
}
"#;
