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

box.screenshot-toolbar {
    background-color: rgba(255, 255, 255, 0.90);
    border: 1px solid rgba(0, 0, 0, 0.10);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
}

button.screenshot-toolbar-btn {
    color: #111115;
}

button.screenshot-toolbar-btn:hover {
    background-color: rgba(0, 0, 0, 0.06);
}

label.capsule-separator {
    color: rgba(0, 0, 0, 0.35);
}

button.color-dot-btn.color-dot-white {
    background-color: #ffffff;
    border: 1px solid #d1d5db;
}

/* ── Dynamic Island Media & Recorder (Light) ─────────────────────── */
popover.media-popover>contents,
popover.recording-popover>contents {
    background-color: rgba(255, 255, 255, 0.98);
    border: 1px solid rgba(0, 0, 0, 0.08);
    color: #111827;
}

.media-popover-app-name {
    color: #111827;
}

.media-popover-title {
    color: #111827;
}

.media-popover-artist {
    color: rgba(0, 0, 0, 0.65);
}

.media-popover-progress-bar trough {
    background-color: rgba(0, 0, 0, 0.08);
}

.media-popover-progress-bar progress {
    background-color: #3b82f6;
}

.recording-header-title {
    color: rgba(0, 0, 0, 0.50);
}

.recording-status-badge {
    background-color: rgba(0, 0, 0, 0.05);
    color: rgba(0, 0, 0, 0.70);
    border: 1px solid rgba(0, 0, 0, 0.08);
}

.recording-hero-timer {
    color: #111827;
}

.recording-settings-card {
    background-color: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.06);
}

.recording-setting-row {
    border-bottom: 1px solid rgba(0, 0, 0, 0.04);
}

.recording-setting-icon {
    color: rgba(0, 0, 0, 0.65);
}

.recording-setting-label {
    color: rgba(0, 0, 0, 0.85);
}

.recording-settings-card dropdown>button {
    background-color: rgba(0, 0, 0, 0.04);
    border: 1px solid rgba(0, 0, 0, 0.08);
    color: #111827;
}

.recording-settings-card dropdown>button:hover {
    background-color: rgba(0, 0, 0, 0.08);
}

.recording-area-btn {
    background-color: rgba(0, 0, 0, 0.05);
    border: 1px solid rgba(0, 0, 0, 0.10);
    color: #111827;
}

.recording-meta-card {
    background-color: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.06);
}

.recording-meta-chip {
    color: rgba(0, 0, 0, 0.85);
}

.recording-popover-btn {
    background-color: rgba(0, 0, 0, 0.04);
    border: 1px solid rgba(0, 0, 0, 0.06);
}

.recording-popover-btn:hover {
    background-color: rgba(0, 0, 0, 0.08);
}

.recording-btn-icon-holder {
    background-color: rgba(0, 0, 0, 0.06);
}

.recording-btn-title {
    color: rgba(0, 0, 0, 0.85);
}

label.recording-capsule-timer {
    color: #111827;
}

/* ── Control Center (Light) ──────────────────────────────────────── */
.control-center-box,
.calendar-box {
    background-color: #ffffff;
    border: 1px solid rgba(0, 0, 0, 0.08);
    color: #111827;
}

.control-center-title,
.calendar-top-date {
    color: #111827;
}

.circle-btn {
    background-color: rgba(0, 0, 0, 0.04);
    border: 1px solid rgba(0, 0, 0, 0.08);
    color: #111827;
}

.circle-btn:hover {
    background-color: rgba(0, 0, 0, 0.08);
}

.control-tile-container {
    background-color: rgba(0, 0, 0, 0.04);
    border: 1px solid rgba(0, 0, 0, 0.06);
}

.control-tile-left-btn:hover,
.control-tile-right-btn:hover {
    background-color: rgba(0, 0, 0, 0.06);
}

.control-tile-left-btn.active {
    background-color: #3b82f6;
    color: #ffffff;
}

.control-tile-left-btn.active .tile-title,
.control-tile-left-btn.active .tile-subtitle {
    color: #ffffff;
}

.control-square-tile {
    background-color: rgba(0, 0, 0, 0.04);
    border: 1px solid rgba(0, 0, 0, 0.06);
    color: rgba(0, 0, 0, 0.75);
}

.control-square-tile:hover {
    background-color: rgba(0, 0, 0, 0.08);
}

.control-square-tile.active {
    background-color: #3b82f6;
    border-color: #2563eb;
    color: #ffffff;
}

.control-slider-card,
.control-disk-card {
    background-color: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.06);
}

.control-slider-title {
    color: rgba(0, 0, 0, 0.55);
}

.control-slider-val {
    color: #111827;
}

scale.pill-slider trough {
    background-color: rgba(0, 0, 0, 0.08);
}

scale.pill-slider highlight {
    background-color: #111827;
}

.slider-overlay-mute-btn:hover,
.slider-popover-btn:hover {
    background-color: rgba(0, 0, 0, 0.08);
}

.control-disk-name {
    color: rgba(0, 0, 0, 0.85);
}

.control-disk-usage {
    color: rgba(0, 0, 0, 0.50);
}

.control-disk-card progressbar trough {
    background-color: rgba(0, 0, 0, 0.06);
}

.control-disk-card progressbar progress {
    background-color: #10b981;
}

/* Control Popovers (Light) */
popover.control-popover>contents,
popover.wifi-popover>contents,
popover.clean-popover>contents,
popover.vpn-popover>contents,
popover.audio-menu-popover>contents {
    background-color: #ffffff;
    border: 1px solid rgba(0, 0, 0, 0.08);
    color: #111827;
}

.audio-menu-section-title {
    color: #3b82f6;
}

.audio-menu-item-btn {
    color: rgba(0, 0, 0, 0.85);
}

.audio-menu-item-btn:hover {
    background-color: rgba(0, 0, 0, 0.05);
}

.audio-menu-item-btn.active {
    background-color: rgba(59, 130, 246, 0.12);
    border: 1px solid rgba(59, 130, 246, 0.35);
    color: #2563eb;
}

.audio-menu-item-check {
    color: #3b82f6;
}

.clean-popover-percent {
    color: #10b981;
}

.clean-popover-size {
    color: #111827;
}

.clean-target-name {
    color: rgba(0, 0, 0, 0.85);
}

.clean-target-size {
    color: rgba(0, 0, 0, 0.50);
}

/* ── Calendar & Notifications (Light) ────────────────────────────── */
.calendar-widget {
    color: #111827;
}

.calendar-widget header {
    color: #111827;
}

.calendar-widget header button:hover {
    background-color: rgba(0, 0, 0, 0.06);
}

.calendar-widget day-name {
    color: rgba(0, 0, 0, 0.45);
}

.calendar-widget day {
    color: rgba(0, 0, 0, 0.85);
}

.calendar-widget day:hover {
    background-color: rgba(0, 0, 0, 0.06);
}

.calendar-widget day.selected {
    background-color: #3b82f6;
    color: #ffffff;
}

.notif-panel-title {
    color: #111827;
}

.clear-all-btn {
    background-color: rgba(0, 0, 0, 0.04);
    border: 1px solid rgba(0, 0, 0, 0.08);
    color: rgba(0, 0, 0, 0.70);
}

.clear-all-btn:hover {
    background-color: rgba(0, 0, 0, 0.08);
    color: #111827;
}

.notif-empty-label {
    color: rgba(0, 0, 0, 0.40);
}

.notif-group-header {
    background-color: rgba(0, 0, 0, 0.03);
}

.notif-stack-item {
    background-color: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.06);
}

.notif-stack-item:hover {
    background-color: rgba(0, 0, 0, 0.06);
}

.notif-item-title {
    color: #111827;
}

.notif-item-body {
    color: rgba(0, 0, 0, 0.70);
}

.notif-item-sub-time {
    color: rgba(0, 0, 0, 0.45);
}
"#;

