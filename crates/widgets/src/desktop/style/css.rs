//! Desktop Shell Styling matching babydra-desktop

pub const DESKTOP_CSS: &str = r#"
.desktop-window {
    background-color: transparent;
    background-image: none;
    border: none;
    box-shadow: none;
}

.desktop-wallpaper-container {
    background-color: #0b0c10;
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

.desktop-icon:hover {
    background-color: rgba(255, 255, 255, 0.14);
    border-color: rgba(255, 255, 255, 0.2);
}

.desktop-icon.selected {
    background-color: rgba(255, 255, 255, 0.26);
    border-color: rgba(255, 255, 255, 0.45);
}

.desktop-icon-image {
    margin-bottom: 3px;
}

.desktop-icon-label {
    color: #ffffff;
    font-size: 12px;
    font-weight: 500;
    line-height: 1.2;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.95), 0 0 2px rgba(0, 0, 0, 0.8);
    padding: 0 2px;
}

.desktop-rubberband {
    background-color: rgba(100, 180, 255, 0.25);
    border: 1px solid rgba(100, 180, 255, 0.7);
    border-radius: 4px;
}
"#;

