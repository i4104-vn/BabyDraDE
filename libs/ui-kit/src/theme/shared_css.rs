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
.control-slider-val,
.control-disk-usage,
.clean-popover-size,
.clean-target-size,
.notif-item-sub-time,
.recording-hero-timer,
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

/* ========================================================================= */
/* Screenshot Editor Styles                                                  */
/* ========================================================================= */
window.screenshot-window,
window.screenshot-window.background {
    background-color: transparent;
    background-image: none;
    box-shadow: none;
    border: none;
}

box.screenshot-toolbar {
    background-image: none;
    border-radius: 9999px;
    padding: 4px 12px;
    box-shadow: none;
}

button.screenshot-toolbar-btn {
    background-image: none;
    border-radius: 9999px;
    padding: 5px 10px;
    margin: 2px;
    transition: all 200ms ease;
    min-height: 24px;
    box-shadow: none;
}

button.screenshot-toolbar-btn:hover {
    background-image: none;
    box-shadow: none;
}

button.screenshot-toolbar-btn:active {
    background-color: rgba(255, 255, 255, 0.16);
    background-image: none;
    box-shadow: none;
}

button.screenshot-toolbar-btn.selected {
    background-color: rgba(59, 130, 246, 0.70);
    background-image: none;
    border: 1px solid rgba(147, 197, 253, 0.25);
    border-top: 1px solid rgba(147, 197, 253, 0.45);
    box-shadow: none;
}

label.capsule-separator {
    font-size: 9px;
    margin: 0 2px;
}

box.color-dot-indicator {
    border-radius: 9999px;
    min-width: 14px;
    min-height: 14px;
    box-shadow: none;
}

popover.screenshot-color-popover > contents {
    padding: 8px;
}

button.color-dot-btn {
    border-radius: 9999px;
    min-width: 16px;
    min-height: 16px;
    margin: 3px;
    padding: 0;
    background-image: none;
    box-shadow: none;
    transition: all 150ms ease;
}

button.color-dot-btn:hover {
    border-color: rgba(255, 255, 255, 0.6);
    box-shadow: none;
}

button.color-dot-btn.color-dot-red {
    background-color: #ef4444;
    background-image: none;
}

button.color-dot-btn.color-dot-orange {
    background-color: #f97316;
    background-image: none;
}

button.color-dot-btn.color-dot-yellow {
    background-color: #eab308;
    background-image: none;
}

button.color-dot-btn.color-dot-green {
    background-color: #22c55e;
    background-image: none;
}

button.color-dot-btn.color-dot-blue {
    background-color: #3b82f6;
    background-image: none;
}

button.color-dot-btn.color-dot-purple {
    background-color: #a855f7;
    background-image: none;
}

button.color-dot-btn.color-dot-white {
    background-image: none;
}

button.color-dot-btn.color-dot-black {
    background-color: #000000;
    background-image: none;
}

/* ========================================================================= */
/* Lock Screen Styles                                                        */
/* ========================================================================= */
@keyframes shake {
    0% { margin-left: 0px; margin-right: 0px; }
    15% { margin-left: -12px; margin-right: 12px; }
    30% { margin-left: 12px; margin-right: -12px; }
    45% { margin-left: -8px; margin-right: 8px; }
    60% { margin-left: 8px; margin-right: -8px; }
    75% { margin-left: -4px; margin-right: 4px; }
    90% { margin-left: 4px; margin-right: -4px; }
    100% { margin-left: 0px; margin-right: 0px; }
}

.lock-window {
    opacity: 1;
    font-family: 'Quicksand', 'Segoe UI Variable Static Text', 'Segoe UI', 'Inter', system-ui, sans-serif;
}

.lock-tint {
    background: radial-gradient(circle at center, rgba(0, 0, 0, 0.25) 0%, rgba(0, 0, 0, 0.65) 100%);
}

.lock-card {
    background-color: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 20px;
    padding: 28px 40px;
    min-width: 360px;
    box-shadow: none;
    font-family: 'Quicksand', 'Segoe UI Variable Static Text', 'Segoe UI', 'Inter', system-ui, sans-serif;
}

.lock-clock {
    font-size: 72px;
    font-weight: 800;
    margin-bottom: 0px;
    text-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    color: #ffffff;
}

.lock-date {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 6px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: rgba(255, 255, 255, 0.7);
}

.lock-avatar {
    background-color: rgba(255, 255, 255, 0.1);
    border: 2px solid rgba(255, 255, 255, 0.25);
    border-radius: 9999px;
    padding: 0px;
    margin-top: 14px;
    margin-bottom: 14px;
    box-shadow: none;
}

.lock-username {
    font-size: 18px;
    font-weight: 700;
    margin-bottom: 12px;
    color: #ffffff;
}

.lock-input {
    background-color: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 30px;
    padding: 10px 20px;
    font-size: 14px;
    transition: all 250ms ease;
    min-width: 280px;
    caret-color: #ffffff;
    color: #ffffff;
    box-shadow: none;
}

.lock-input:focus {
    background-color: rgba(255, 255, 255, 0.12);
    border: 1px solid rgba(255, 255, 255, 0.4);
    box-shadow: none;
}

.lock-status {
    font-size: 12px;
    font-weight: 500;
    margin-top: 16px;
    transition: color 200ms ease;
    color: rgba(255, 255, 255, 0.6);
}

.lock-status.error {
    color: #ff6b6b;
    font-weight: 600;
}

.shake-error .lock-input {
    border-color: #ff6b6b;
    background-color: rgba(255, 107, 107, 0.05);
}

.shake-error {
    animation-name: shake;
    animation-duration: 350ms;
    animation-iteration-count: 1;
    animation-timing-function: ease-in-out;
}

/* ── Media Popover in Dynamic Island ────────────────────────────── */
popover.media-popover>contents {
    border-radius: 18px;
    padding: 14px 18px;
}

.media-popover-box {
    min-width: 240px;
}

.media-popover-header {
    margin-bottom: 6px;
}

.media-popover-app-name {
    font-size: 13px;
    font-weight: 700;
}

.media-popover-title {
    font-size: 14px;
    font-weight: 700;
}

.media-popover-artist {
    font-size: 12px;
    font-weight: 500;
}

.media-popover-progress-bar trough {
    min-height: 5px;
    border-radius: 9999px;
}

.media-popover-progress-bar progress {
    border-radius: 9999px;
}

.media-popover-controls {
    margin-top: 6px;
}

.play-pause-btn {
    min-width: 34px;
    min-height: 34px;
}

/* ── Screen Recorder in Dynamic Island & Popover ─────────────────── */
popover.recording-popover {
    background-color: transparent;
    padding: 0;
    margin: 0;
    box-shadow: none;
    border: none;
}

popover.recording-popover>contents {
    border-radius: 20px;
    padding: 16px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.60);
}

.recording-popover-box {
    min-width: 320px;
}

.recording-popover-header {
    margin-bottom: 6px;
}

.recording-header-icon {
    color: #ef4444;
    opacity: 0.90;
}

.recording-header-title {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 1px;
}

.recording-status-badge {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.8px;
    padding: 3px 10px;
    border-radius: 9999px;
}

.recording-status-badge.badge-ready,
.recording-status-badge.badge-idle {
    background-color: rgba(16, 185, 129, 0.16);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.35);
}

.recording-status-badge.badge-recording {
    background-color: rgba(239, 68, 68, 0.20);
    color: #ef4444;
    border: 1px solid rgba(239, 68, 68, 0.40);
}

.recording-status-badge.badge-paused {
    background-color: rgba(234, 179, 8, 0.20);
    color: #eab308;
    border: 1px solid rgba(234, 179, 8, 0.40);
}

.recording-timer-container {
    margin: 4px 0 12px 0;
}

.recording-hero-timer {
    font-size: 32px;
    font-weight: 800;
    font-family: "JetBrains Mono", monospace;
    letter-spacing: 2px;
}

.recording-settings-card {
    border-radius: 14px;
    padding: 6px 12px;
}

.recording-setting-row {
    min-height: 38px;
    padding: 3px 2px;
}

.recording-setting-icon {
    opacity: 0.65;
}

.recording-setting-label {
    font-size: 12px;
    font-weight: 500;
}

.recording-settings-card dropdown {
    background: transparent;
    border: none;
    box-shadow: none;
}

.recording-settings-card dropdown>button {
    border-radius: 9px;
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 500;
    box-shadow: none;
    min-width: 130px;
    transition: all 150ms ease;
}

.recording-area-btn {
    border-radius: 8px;
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 600;
    transition: all 150ms ease;
}

.recording-area-label {
    font-size: 12px;
    opacity: 0.65;
    font-family: monospace;
}

.recording-action-row {
    margin-top: 8px;
}

.recording-meta-card {
    border-radius: 12px;
    padding: 10px 14px;
    margin: 10px 0;
}

.recording-meta-chip {
    font-size: 12px;
    font-weight: 600;
}

.recording-popover-btn {
    border-radius: 14px;
    padding: 8px 4px;
    transition: all 180ms ease;
}

.recording-btn-icon-holder {
    border-radius: 9999px;
    margin-bottom: 2px;
}

.recording-btn-title {
    font-size: 12px;
    font-weight: 600;
}

.recording-action-stop .recording-btn-icon-holder {
    background-color: rgba(239, 68, 68, 0.20);
    color: #ef4444;
}

.recording-action-stop:hover {
    background-color: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.35);
}

.recording-popover-btn.alert-active {
    background-color: rgba(239, 68, 68, 0.16);
    border-color: rgba(239, 68, 68, 0.40);
}

.recording-popover-btn.alert-active .recording-btn-icon-holder {
    background-color: rgba(239, 68, 68, 0.30);
    color: #fca5a5;
}

.recording-popover-btn.alert-active .recording-btn-title {
    color: #fca5a5;
}

window.recorder-window {
    border-radius: 18px;
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.45);
}

button.active-mode {
    background-color: #3b82f6;
    color: #ffffff;
    font-weight: 700;
    border-radius: 8px;
}

label.recorder-timer {
    font-size: 24px;
    font-weight: 700;
    font-family: monospace;
    letter-spacing: 2px;
    margin: 8px 0;
}

button.recorder-action-btn {
    font-weight: 700;
    font-size: 14px;
    min-height: 40px;
    border-radius: 12px;
}

button.recorder-action-btn.destructive-action {
    background-color: #ef4444;
    color: #ffffff;
}

button.recorder-action-btn.destructive-action:hover {
    background-color: #dc2626;
}

label.dim-label {
    opacity: 0.65;
    font-size: 12px;
}

.recording-capsule-icon {
    color: #ef4444;
    opacity: 0.90;
}

label.recording-capsule-timer {
    font-size: 12px;
    font-weight: 700;
    font-family: monospace;
    letter-spacing: 0.5px;
    margin: 0 4px;
}

/* ── Control Center Window & Layout ─────────────────────────────── */
window.control-center-window,
window.calendar-window {
    background-color: transparent;
    background-image: none;
    box-shadow: none;
    border: none;
}

.control-center-blur-host,
.calendar-blur-host {
    border-radius: 20px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.65);
    transition: opacity 250ms ease, transform 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.control-center-box,
.calendar-box {
    border-radius: 20px;
    padding: 16px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-top: 1px solid rgba(255, 255, 255, 0.22);
}

.control-center-title {
    font-size: 16px;
    font-weight: 800;
    letter-spacing: 0.5px;
}

.circle-btn {
    border-radius: 9999px;
    min-width: 32px;
    min-height: 32px;
    padding: 6px;
    transition: background-color 150ms ease, color 150ms ease;
}

/* Quick Toggle Tiles */
.control-center-grid {
    min-height: 120px;
}

.control-left-toggles-box {
    min-width: 140px;
}

.control-tile-container {
    border-radius: 14px;
    min-height: 56px;
    transition: background-color 150ms ease, border-color 150ms ease;
}

.control-tile-left-btn {
    border-radius: 14px 0 0 14px;
    padding: 8px 12px;
    min-height: 56px;
    transition: background-color 150ms ease;
}

.control-tile-right-btn {
    border-radius: 0 14px 14px 0;
    padding: 8px 10px;
    min-height: 56px;
    min-width: 32px;
    border-left: 1px solid rgba(255, 255, 255, 0.08);
    transition: background-color 150ms ease;
}

.control-square-tile {
    border-radius: 14px;
    min-width: 56px;
    min-height: 56px;
    transition: all 180ms ease;
}

/* Control Sliders (PillSlider) */
.control-slider-card {
    border-radius: 16px;
    padding: 10px 14px;
}

.control-slider-title {
    font-size: 13px;
    font-weight: 700;
}

.control-slider-val {
    font-size: 13px;
    font-weight: 700;
    font-feature-settings: "tnum";
    font-variant-numeric: tabular-nums;
}

.pill-slider-container {
    min-height: 32px;
}

scale.pill-slider {
    min-height: 32px;
    margin: 0;
    padding: 0;
}

scale.pill-slider trough {
    min-height: 32px;
    border-radius: 9999px;
    border: none;
}

scale.pill-slider highlight {
    min-height: 32px;
    border-radius: 9999px;
    border: none;
}

scale.pill-slider slider {
    min-width: 0;
    min-height: 0;
    opacity: 0;
    border: none;
    box-shadow: none;
}

.slider-overlay-icon {
    margin-left: 10px;
}

.slider-overlay-mute-btn {
    border-radius: 9999px;
    min-width: 28px;
    min-height: 28px;
    margin-left: 6px;
    transition: background-color 150ms ease;
}

.slider-popover-btn {
    border-radius: 9999px;
    min-width: 28px;
    min-height: 28px;
    padding: 4px;
    transition: background-color 150ms ease;
}

/* Storage Card */
.control-disk-card {
    border-radius: 16px;
    padding: 10px 14px;
}

.control-disk-item {
    margin-top: 4px;
}

.control-disk-title-box {
    margin-bottom: 3px;
}

.control-disk-name {
    font-size: 12px;
    font-weight: 600;
}

.control-disk-usage {
    font-size: 11.5px;
    font-weight: 500;
    font-feature-settings: "tnum";
}

.control-disk-card progressbar trough {
    min-height: 6px;
    border-radius: 9999px;
    border: none;
}

.control-disk-card progressbar progress {
    border-radius: 9999px;
}

/* Control Popovers */
popover.control-popover>contents,
popover.wifi-popover>contents,
popover.clean-popover>contents,
popover.vpn-popover>contents,
popover.audio-menu-popover>contents {
    border-radius: 18px;
    padding: 14px 16px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.65);
}

.audio-menu-section-title {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.8px;
    text-transform: uppercase;
    margin-bottom: 6px;
}

.audio-menu-item-btn {
    border-radius: 10px;
    padding: 8px 12px;
    min-height: 36px;
    transition: background-color 150ms ease;
}

.audio-menu-item-check {
    font-size: 13px;
    font-weight: 700;
    margin-left: 6px;
}

.clean-popover-percent {
    font-size: 24px;
    font-weight: 800;
    letter-spacing: 1px;
}

.clean-popover-size {
    font-size: 13px;
    font-weight: 600;
    font-feature-settings: "tnum";
}

.clean-target-list {
    margin-top: 6px;
    margin-bottom: 6px;
}

.clean-target-row {
    padding: 3px 0;
}

.clean-target-bullet {
    font-size: 14px;
    margin-right: 4px;
    color: #ef4444;
}

.clean-target-name {
    font-size: 12.5px;
    font-weight: 500;
}

.clean-target-size {
    font-size: 12.5px;
    font-weight: 600;
    font-feature-settings: "tnum";
}

.clean-popover-btn {
    border-radius: 9999px;
    min-height: 34px;
    font-weight: 700;
    font-size: 13px;
}

/* ── Calendar & Notifications ───────────────────────────────────── */
.calendar-top-row {
    margin-bottom: 6px;
}

.calendar-top-date {
    font-size: 16px;
    font-weight: 800;
}

.calendar-widget {
    border: none;
    background: transparent;
    font-size: 12px;
}

.calendar-widget header {
    border: none;
    background: transparent;
    padding: 4px;
}

.calendar-widget header button {
    border-radius: 9999px;
    min-width: 28px;
    min-height: 28px;
    transition: background-color 150ms ease;
}

.calendar-widget grid {
    border: none;
}

.calendar-widget day-name {
    font-weight: 700;
    font-size: 11px;
    padding: 4px;
}

.calendar-widget day {
    border-radius: 9999px;
    min-width: 30px;
    min-height: 30px;
    margin: 2px;
    transition: background-color 150ms ease;
}

.notif-panel-title {
    font-size: 14px;
    font-weight: 700;
}

.clear-all-btn {
    font-size: 12px;
    font-weight: 600;
    border-radius: 8px;
    padding: 3px 10px;
    transition: background-color 150ms ease;
}

.notif-scroll {
    border: none;
    background: transparent;
}

.notif-stack-box {
    padding: 2px 0;
}

.notif-empty-label {
    font-size: 13px;
    font-weight: 500;
    padding: 36px 0;
}

.notif-group-container {
    margin-bottom: 8px;
}

.notif-group-header {
    padding: 6px 8px;
    border-radius: 10px;
    margin-bottom: 4px;
}

.notif-sub-box {
    margin-top: 4px;
}

.notif-stack-item {
    border-radius: 14px;
    padding: 10px 12px;
    margin-bottom: 6px;
    transition: background-color 150ms ease;
}

.notif-item-icon {
    margin-right: 6px;
}

.notif-item-title {
    font-size: 13px;
    font-weight: 700;
}

.notif-item-body {
    font-size: 12px;
    font-weight: 450;
    line-height: 1.3;
}

.notif-item-sub-time {
    font-size: 11px;
    font-weight: 500;
    font-feature-settings: "tnum";
}

.notif-count-badge {
    border-radius: 9999px;
    font-size: 10px;
    font-weight: 700;
    padding: 1px 6px;
}
"#;

