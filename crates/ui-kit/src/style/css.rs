//! Reusable Fluent UI CSS Stylesheet

pub const UI_KIT_BASE_CSS: &str = r#"
/* BabyDraDE UI Kit Reusable Stylesheet */

* {
    font-family: 'Quicksand', 'Segoe UI Variable Static Text', 'Segoe UI', 'Inter', system-ui, sans-serif;
    -gtk-icon-style: symbolic;
}

/* Window transparent background */
window {
    background: transparent;
    background-color: transparent;
}

/* Global button reset to prevent Adwaita default borders/outlines/shadows from causing hover shifts */
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

/* =============================================================================
   Reusable Pill Container
   ============================================================================= */
.pill-container {
    background-color: rgb(14, 14, 18);
    background-image: none;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    box-shadow: none;
    padding: 2px 8px;
    min-height: 26px;
    color: #F3F4F6;
    transition: background-color 200ms ease, border-color 200ms ease;
}

.pill-container:hover {
    box-shadow: none;
}

/* Vertical Separator */
.separator,
.capsule-separator {
    color: rgba(255, 255, 255, 0.14);
    margin: 0 4px;
    font-weight: 300;
    font-size: 9px;
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
    background-color: transparent;
    background-image: none;
    border: none;
    border-radius: 6px;
    padding: 2px 6px;
    margin: 0;
    min-height: 20px;
    color: #D1D5DB;
    font-size: 12.5px;
    font-weight: 450;
    transition: background-color 120ms ease, color 120ms ease;
}

.fluent-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
    border: none;
    box-shadow: none;
    color: #FFFFFF;
}

.fluent-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
    border: none;
    box-shadow: none;
}

/* Arch Logo Button */
.arch-btn,
button.panel-logo-btn {
    background-color: transparent;
    background-image: none;
    border: none;
    padding: 2px 4px;
    margin: 0;
    border-radius: 6px;
    min-height: 20px;
    min-width: 20px;
    box-shadow: none;
    transition: background-color 150ms ease;
}

.arch-btn:hover,
button.panel-logo-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
    border: none;
    box-shadow: none;
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

/* =============================================================================
   Context Menu Overlay (Matching babydra-core & explore)
   ============================================================================= */
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
    background: none;
    border: none;
}

.context-menu-box {
    background-color: rgb(14, 14, 18);
    background-image: none;
    opacity: 1.0;
    color: #e8e8e8;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.4);
    padding: 4px;
    animation: context-menu-pop 160ms cubic-bezier(0.16, 1, 0.3, 1);
}

button.context-menu-item {
    background-image: none;
    background-color: transparent;
    border: none;
    outline: none;
    box-shadow: none;
    border-radius: 6px;
    padding: 6px 12px;
    font-size: 14px;
    min-height: 30px;
    color: #e8e8e8;
    transition: background-color 140ms ease, color 140ms ease, padding-left 140ms ease;
}

button.context-menu-item:focus,
button.context-menu-item:hover {
    background-color: rgba(255, 255, 255, 0.08);
    background-image: none;
    color: #ffffff;
    outline: none;
    box-shadow: none;
    padding-left: 14px;
}

button.context-menu-item label {
    color: #e8e8e8;
}

button.context-menu-item:focus label,
button.context-menu-item:hover label {
    color: #ffffff;
}

button.context-menu-item.destructive {
    color: #ef4444;
}

button.context-menu-item.destructive label {
    color: #ef4444;
}

button.context-menu-item.destructive:hover {
    background-color: rgba(239, 68, 68, 0.15);
}

button.context-menu-item.destructive:hover label {
    color: #ef4444;
}

.shortcut-label {
    font-size: 12px;
    opacity: 0.55;
    margin-left: 12px;
    transition: opacity 140ms ease;
}

button.context-menu-item:hover .shortcut-label {
    opacity: 0.85;
}

.context-menu-separator,
.menu-sep {
    margin-top: 3px;
    margin-bottom: 3px;
    min-height: 1px;
    background-color: rgba(255, 255, 255, 0.10);
    opacity: 1;
}

.context-menu-header,
.group-header-label {
    font-weight: 700;
    font-size: 12px;
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
    background-image: none;
    background-color: transparent;
    border: none;
    outline: none;
    box-shadow: none;
    border-radius: 6px;
    padding: 4px;
    margin: 2px;
    min-width: 26px;
    min-height: 26px;
    color: #e8e8e8;
    transition: background-color 140ms ease, color 140ms ease;
}

button.context-menu-footer-btn:hover,
button.context-menu-footer-btn:active {
    background-color: rgba(255, 255, 255, 0.08);
    background-image: none;
    color: #ffffff;
    border: none;
    outline: none;
    box-shadow: none;
}

button.context-menu-footer-btn:focus {
    background-image: none;
    background-color: transparent;
    color: #e8e8e8;
    border: none;
    outline: none;
    box-shadow: none;
}

/* =============================================================================
   Popovers & Tooltip Popover Overlay (Matching babydra-core)
   ============================================================================= */
popover>contents,
popover.menu>contents,
popover.status-popover>contents,
popover.tooltip-popover>contents {
    background-color: rgb(14, 14, 18);
    background-image: none;
    opacity: 1.0;
    color: #e8e8e8;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 14px;
    padding: 10px 14px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.50);
}

.status-popover-card,
.tooltip-popover-card {
    min-width: 200px;
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
    margin-bottom: 4px;
    color: #ffffff;
}

.status-popover-sep,
.tooltip-popover-sep {
    opacity: 0.15;
    margin-bottom: 6px;
    background-color: rgba(255, 255, 255, 0.20);
}

.status-popover-key,
.tooltip-popover-key {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.70);
}

.status-popover-val,
.tooltip-popover-val {
    font-size: 12px;
    font-weight: 600;
    color: #ffffff;
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
"#;
