pub const TOPBAR_CSS: &str = r#"
/* BabyDraDE TopBar Fluent Dark Stylesheet */

window {
    background: transparent;
}

.topbar-box {
    background: transparent;
    padding: 6px 12px 0px 12px;
}

/* Base pill styling */
.pill-container {
    background-color: rgba(24, 26, 32, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.4);
    padding: 4px 12px;
    color: #F3F4F6;
    font-family: 'Segoe UI Variable', 'Segoe UI', system-ui, -apple-system, sans-serif;
    font-size: 13px;
    transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
}

.pill-container:hover {
    border-color: rgba(255, 255, 255, 0.22);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.5);
}

/* Vertical Separator */
.separator {
    color: rgba(255, 255, 255, 0.22);
    margin: 0 6px;
    font-weight: 300;
}

/* Left Section */
.arch-btn {
    background: transparent;
    border: none;
    padding: 0 4px;
    border-radius: 6px;
    color: #1793D1;
    font-weight: 700;
    transition: transform 150ms ease;
}

.arch-btn:hover {
    transform: scale(1.1);
}

.app-title {
    font-weight: 600;
    color: #FFFFFF;
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

/* Dynamic Island (Center) */
.dynamic-island {
    background-color: rgba(16, 18, 24, 0.92);
    border: 1px solid rgba(56, 185, 240, 0.30);
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.55), 0 0 12px rgba(56, 185, 240, 0.10);
    padding: 3px 14px;
}

.dynamic-island:hover {
    border-color: rgba(56, 185, 240, 0.55);
    box-shadow: 0 6px 22px rgba(0, 0, 0, 0.65), 0 0 16px rgba(56, 185, 240, 0.20);
}

.island-disc {
    background-color: #1A1F2C;
    border: 1px solid rgba(255, 255, 255, 0.25);
    border-radius: 9999px;
    padding: 2px 5px;
    color: #38B9F0;
    font-size: 11px;
}

.island-title {
    font-weight: 600;
    color: #F9FAFB;
    margin: 0 10px;
    font-size: 12.5px;
}

.island-badge {
    background: rgba(56, 185, 240, 0.20);
    color: #38B9F0;
    border-radius: 9999px;
    padding: 1px 7px;
    font-weight: 700;
    font-size: 11px;
}

/* Right Section Pills */
.tray-pill {
    padding: 4px 10px;
}

.tray-icon {
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 2px 4px;
    color: #9CA3AF;
    font-size: 13px;
    transition: color 120ms ease, background 120ms ease;
}

.tray-icon:hover {
    background: rgba(255, 255, 255, 0.10);
    color: #FFFFFF;
}

.status-pill {
    padding: 4px 12px;
}

.status-icon {
    font-size: 13.5px;
    color: #E5E7EB;
    margin: 0 3px;
}

.clock-label {
    font-weight: 650;
    font-size: 13px;
    color: #FFFFFF;
    letter-spacing: 0.3px;
}

/* Popover menu styling */
popover.menu {
    background-color: rgba(24, 26, 32, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 12px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
    padding: 6px;
}

popover button {
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 4px 10px;
    color: #E5E7EB;
    text-align: left;
    font-size: 12.5px;
}

popover button:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #FFFFFF;
}
"#;

