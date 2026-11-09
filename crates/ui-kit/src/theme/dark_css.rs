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

/* ── Dynamic Island Media & Recorder (Dark) ──────────────────────── */
popover.media-popover>contents,
popover.recording-popover>contents {
    background-color: rgba(14, 14, 18, 0.98);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-top: 1px solid rgba(255, 255, 255, 0.22);
    color: #ffffff;
}

.media-popover-app-name {
    color: #ffffff;
}

.media-popover-title {
    color: #ffffff;
}

.media-popover-artist {
    color: rgba(255, 255, 255, 0.75);
}

.media-popover-progress-bar trough {
    background-color: rgba(255, 255, 255, 0.10);
}

.media-popover-progress-bar progress {
    background-color: #3b82f6;
}

.recording-header-title {
    color: rgba(255, 255, 255, 0.50);
}

.recording-status-badge {
    background-color: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.70);
    border: 1px solid rgba(255, 255, 255, 0.12);
}

.recording-hero-timer {
    color: #ffffff;
}

.recording-settings-card {
    background-color: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
}

.recording-setting-row {
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.recording-setting-icon {
    color: rgba(255, 255, 255, 0.75);
}

.recording-setting-label {
    color: rgba(255, 255, 255, 0.85);
}

.recording-settings-card dropdown>button {
    background-color: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #ffffff;
}

.recording-settings-card dropdown>button:hover {
    background-color: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.22);
}

.recording-area-btn {
    background-color: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.14);
    color: #ffffff;
}

.recording-area-btn:hover {
    background-color: rgba(255, 255, 255, 0.14);
    border-color: rgba(255, 255, 255, 0.22);
}

.recording-meta-card {
    background-color: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
}

.recording-meta-chip {
    color: rgba(255, 255, 255, 0.85);
}

.recording-popover-btn {
    background-color: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
}

.recording-popover-btn:hover {
    background-color: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.18);
}

.recording-btn-icon-holder {
    background-color: rgba(255, 255, 255, 0.08);
}

.recording-btn-title {
    color: rgba(255, 255, 255, 0.85);
}

label.recording-capsule-timer {
    color: #ffffff;
}

/* ── Control Center (Dark) ───────────────────────────────────────── */
.control-center-box,
.calendar-box {
    background-color: rgb(14, 14, 18);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-top: 1px solid rgba(255, 255, 255, 0.24);
    color: #ffffff;
}

.control-center-title,
.calendar-top-date {
    color: #ffffff;
}

.circle-btn {
    background-color: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #ffffff;
}

.circle-btn:hover {
    background-color: rgba(255, 255, 255, 0.14);
    border-color: rgba(255, 255, 255, 0.22);
}

.control-tile-container {
    background-color: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.10);
}

.control-tile-left-btn:hover,
.control-tile-right-btn:hover {
    background-color: rgba(255, 255, 255, 0.08);
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
    background-color: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.10);
    color: rgba(255, 255, 255, 0.85);
}

.control-square-tile:hover {
    background-color: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.18);
}

.control-square-tile.active {
    background-color: #3b82f6;
    border-color: #60a5fa;
    color: #ffffff;
}

.control-slider-card,
.control-disk-card {
    background-color: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
}

.control-slider-title {
    color: rgba(255, 255, 255, 0.65);
}

.control-slider-val {
    color: #ffffff;
}

scale.pill-slider trough {
    background-color: rgba(255, 255, 255, 0.10);
}

scale.pill-slider highlight {
    background-color: rgba(255, 255, 255, 0.90);
}

.slider-overlay-mute-btn:hover,
.slider-popover-btn:hover {
    background-color: rgba(255, 255, 255, 0.12);
}

.control-disk-name {
    color: rgba(255, 255, 255, 0.85);
}

.control-disk-usage {
    color: rgba(255, 255, 255, 0.55);
}

.control-disk-card progressbar trough {
    background-color: rgba(255, 255, 255, 0.08);
}

.control-disk-card progressbar progress {
    background-color: #10b981;
}

/* Control Popovers (Dark) */
popover.control-popover>contents,
popover.wifi-popover>contents,
popover.clean-popover>contents,
popover.vpn-popover>contents,
popover.audio-menu-popover>contents {
    background-color: rgb(14, 14, 18);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #ffffff;
}

.audio-menu-section-title {
    color: #60a5fa;
}

.audio-menu-item-btn {
    color: rgba(255, 255, 255, 0.90);
}

.audio-menu-item-btn:hover {
    background-color: rgba(255, 255, 255, 0.08);
    color: #ffffff;
}

.audio-menu-item-btn.active {
    background-color: rgba(59, 130, 246, 0.20);
    border: 1px solid rgba(59, 130, 246, 0.50);
    color: #60a5fa;
}

.audio-menu-item-check {
    color: #3b82f6;
}

.clean-popover-percent {
    color: #10b981;
}

.clean-popover-size {
    color: #ffffff;
}

.clean-target-name {
    color: rgba(255, 255, 255, 0.85);
}

.clean-target-size {
    color: rgba(255, 255, 255, 0.50);
}

/* ── Calendar & Notifications (Dark) ─────────────────────────────── */
.calendar-widget {
    color: #ffffff;
}

.calendar-widget header {
    color: #ffffff;
}

.calendar-widget header button:hover {
    background-color: rgba(255, 255, 255, 0.10);
}

.calendar-widget day-name {
    color: rgba(255, 255, 255, 0.45);
}

.calendar-widget day {
    color: rgba(255, 255, 255, 0.90);
}

.calendar-widget day:hover {
    background-color: rgba(255, 255, 255, 0.10);
}

.calendar-widget day.selected {
    background-color: #3b82f6;
    color: #ffffff;
}

.notif-panel-title {
    color: #ffffff;
}

.clear-all-btn {
    background-color: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.10);
    color: rgba(255, 255, 255, 0.80);
}

.clear-all-btn:hover {
    background-color: rgba(255, 255, 255, 0.12);
    color: #ffffff;
}

.notif-empty-label {
    color: rgba(255, 255, 255, 0.40);
}

.notif-group-header {
    background-color: rgba(255, 255, 255, 0.04);
}

.notif-stack-item {
    background-color: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
}

.notif-stack-item:hover {
    background-color: rgba(255, 255, 255, 0.09);
    border-color: rgba(255, 255, 255, 0.14);
}

.notif-item-title {
    color: #ffffff;
}

.notif-item-body {
    color: rgba(255, 255, 255, 0.75);
}

.notif-item-sub-time {
    color: rgba(255, 255, 255, 0.45);
}
"#;

