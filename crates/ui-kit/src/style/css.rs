//! Reusable Fluent UI CSS Stylesheet

pub const UI_KIT_BASE_CSS: &str = r#"
/* BabyDraDE UI Kit Reusable Stylesheet */

/* Window transparent background */
window {
    background: transparent;
}

/* Reusable Pill Container */
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

/* Reusable Circular Disc / Vinyl */
.circular-disc {
    background-color: #1A1F2C;
    border: 1px solid rgba(255, 255, 255, 0.25);
    border-radius: 9999px;
    padding: 2px 6px;
    color: #38B9F0;
    font-size: 11px;
    min-width: 20px;
    min-height: 20px;
}

/* Reusable Badge / Tag */
.badge {
    background: rgba(56, 185, 240, 0.20);
    color: #38B9F0;
    border-radius: 9999px;
    padding: 1px 7px;
    font-weight: 700;
    font-size: 11px;
}

.badge-subtle {
    background: rgba(255, 255, 255, 0.10);
    color: #E5E7EB;
}

/* Interactive Fluent Button */
.fluent-btn {
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 2px 7px;
    color: #D1D5DB;
    font-size: 12.5px;
    font-weight: 450;
    transition: background 120ms ease, color 120ms ease, transform 120ms ease;
}

.fluent-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #FFFFFF;
}

.fluent-btn:active {
    background: rgba(255, 255, 255, 0.20);
    transform: scale(0.98);
}

/* Arch Logo Button */
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

/* Dynamic Island Capsule */
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

/* Popovers & Flyouts */
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

