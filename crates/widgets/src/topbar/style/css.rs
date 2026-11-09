//! TopBar-specific CSS styling matching babydra-panel Fluent glassmorphism

pub const TOPBAR_CSS: &str = r#"
.topbar-box {
    background: transparent;
    padding: 4px 8px 0px 8px;
}

/* ── Left Pill & App Menu ────────────────────────────────────────── */
.left-pill {
    background: rgba(28, 28, 30, 0.72);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    padding: 3px 8px;
}

.panel-logo-btn {
    background: transparent;
    border: none;
    padding: 2px 4px;
    border-radius: 8px;
    transition: background 150ms ease;
}

.panel-logo-btn:hover {
    background: rgba(255, 255, 255, 0.12);
}

.app-title {
    font-weight: 650;
    color: #FFFFFF;
    font-size: 13px;
    margin-right: 6px;
}

.func-btn {
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 2px 7px;
    color: #D1D5DB;
    font-size: 12.5px;
    font-weight: 450;
    transition: background 120ms ease, color 120ms ease;
}

.func-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #FFFFFF;
}

.func-btn:active {
    background: rgba(255, 255, 255, 0.20);
}

.more-btn {
    font-weight: 700;
    letter-spacing: 1px;
}

/* ── Dynamic Island ──────────────────────────────────────────────── */
.island-wrapper {
    background: transparent;
}

.island-bracket {
    font-weight: 300;
    color: rgba(255, 255, 255, 0.35);
    font-size: 16px;
    margin: 0 2px;
}

.panel-notch {
    background: rgba(20, 20, 24, 0.88);
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 14px;
    padding: 3px 14px;
    min-height: 28px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
    transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
}

.panel-notch:hover {
    background: rgba(28, 28, 34, 0.94);
    border-color: rgba(255, 255, 255, 0.22);
}

.island-title {
    font-weight: 600;
    color: #F9FAFB;
    margin: 0 8px;
    font-size: 12.5px;
}

/* ── Right Section ───────────────────────────────────────────────── */
.capsule-separator {
    color: rgba(255, 255, 255, 0.18);
    font-size: 14px;
    margin: 0 2px;
}

.panel-tray-box {
    background: rgba(28, 28, 30, 0.72);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    padding: 2px 6px;
}

.panel-tray-item-btn {
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 2px 4px;
    color: #E5E7EB;
    transition: background 120ms ease;
}

.panel-tray-item-btn:hover {
    background: rgba(255, 255, 255, 0.12);
}

.sys-monitor-box {
    background: rgba(28, 28, 30, 0.72);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    padding: 3px 10px;
}

.status-text {
    font-size: 12px;
    font-weight: 550;
    color: #E5E7EB;
    font-feature-settings: "tnum";
}

.sys-monitor-popover > contents {
    background-color: rgba(18, 18, 22, 0.96);
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 16px;
    padding: 14px;
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.55);
}

.sys-chart {
    background-color: rgba(0, 0, 0, 0.35);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
}

.status-indicators-box {
    background: rgba(28, 28, 30, 0.72);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    padding: 2px 6px;
}

.panel-status-btn {
    background: transparent;
    border: none;
    padding: 2px 6px;
    border-radius: 8px;
    transition: background 120ms ease;
}

.panel-status-btn:hover {
    background: rgba(255, 255, 255, 0.10);
}

.status-icon {
    margin: 0 4px;
}

.panel-clock-btn {
    background: rgba(28, 28, 30, 0.72);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    padding: 3px 10px;
    transition: background 120ms ease;
}

.panel-clock-btn:hover {
    background: rgba(255, 255, 255, 0.14);
}

.panel-clock {
    font-size: 12.5px;
    font-weight: 600;
    color: #FFFFFF;
    letter-spacing: 0.3px;
    font-feature-settings: "tnum";
}

.clock-bell-box {
    margin-left: 2px;
}

.clock-bell-icon {
    opacity: 0.85;
}

.clock-bell-dot {
    background-color: #ef4444;
    border-radius: 9999px;
    min-width: 6px;
    min-height: 6px;
    margin-top: 1px;
    margin-right: 1px;
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

.speed-high {
    color: #10b981;
}

.speed-medium {
    color: #3b82f6;
}

.speed-low {
    color: rgba(255, 255, 255, 0.75);
}
"#;
