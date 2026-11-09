//! TopBar-specific CSS styling that complements the UI Kit base styles

pub const TOPBAR_CSS: &str = r#"
.topbar-box {
    background: transparent;
    padding: 6px 12px 0px 12px;
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

.island-title {
    font-weight: 600;
    color: #F9FAFB;
    margin: 0 10px;
    font-size: 12.5px;
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
"#;
