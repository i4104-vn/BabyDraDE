//! Reusable Fluent & Obsidian UI CSS Stylesheet matching babydra-core design system

pub const UI_KIT_BASE_CSS: &str = r#"
/* BabyDraDE UI Kit Reusable Stylesheet */

* {
    font-family: 'Quicksand', 'Segoe UI Variable Static Text', 'Segoe UI', 'Inter', system-ui, -apple-system, sans-serif;
    -gtk-icon-style: symbolic;
}

/* Window transparent background */
window {
    background: transparent;
    background-color: transparent;
    box-shadow: none;
    border: none;
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
   Reusable Pill Container (Capsule)
   ============================================================================= */
.pill-container {
    background-color: rgb(14, 14, 18);
    background-image: none;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    box-shadow: none;
    padding: 2px 8px;
    min-height: 26px;
    color: rgba(255, 255, 255, 0.95);
    transition: background-color 200ms ease, border-color 200ms ease;
}

.pill-container:hover {
    background-color: rgb(24, 24, 28);
    border-color: rgba(255, 255, 255, 0.16);
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

/* Reusable Circular Disc */
.circular-disc {
    background-color: rgb(24, 25, 31);
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 9999px;
    padding: 2px 6px;
    color: #3b82f6;
    font-size: 11px;
    min-width: 20px;
    min-height: 20px;
}

/* Reusable Badge / Tag */
.badge {
    background: rgba(59, 130, 246, 0.18);
    color: #60a5fa;
    border-radius: 9999px;
    padding: 1px 7px;
    font-weight: 700;
    font-size: 11px;
}

.badge-subtle {
    background: rgba(255, 255, 255, 0.10);
    color: rgba(255, 255, 255, 0.85);
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
    color: rgba(255, 255, 255, 0.90);
    font-size: 12.5px;
    font-weight: 500;
    transition: background-color 150ms ease, color 150ms ease;
}

.fluent-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
    border: none;
    box-shadow: none;
    color: #ffffff;
}

.fluent-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
    border: none;
    box-shadow: none;
}

/* Standard Button (.baby-button) & Primary Action (.btn-primary) */
button.baby-button {
    border-radius: 10px;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 600;
    background-image: none;
    box-shadow: none;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background-color: rgba(255, 255, 255, 0.06);
    color: #ffffff;
    transition: background-color 200ms ease, border-color 200ms ease, color 200ms ease;
}

button.baby-button:hover {
    background-color: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.20);
    color: #ffffff;
}

button.btn-primary {
    border-radius: 9999px;
    background-color: #3b82f6;
    background-image: none;
    border: none;
    color: #ffffff;
    font-size: 14px;
    font-weight: 600;
    padding: 8px 18px;
    box-shadow: none;
    transition: background-color 150ms ease;
}

button.btn-primary:hover {
    background-color: #2563eb;
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
    color: #ffffff;
    box-shadow: none;
    transition: background-color 150ms ease;
}

.arch-btn:hover,
button.panel-logo-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
    border: none;
    box-shadow: none;
}

.arch-btn:active,
button.panel-logo-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
    border: none;
    box-shadow: none;
}

/* Dynamic Island Capsule */
.dynamic-island {
    background-color: rgb(14, 14, 18);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    box-shadow: none;
    padding: 0 10px;
    min-height: 26px;
    transition: all 400ms cubic-bezier(0.16, 1, 0.3, 1);
}

.dynamic-island:hover {
    background-color: rgb(24, 24, 28);
    border-color: rgba(255, 255, 255, 0.16);
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
    display: none;
    background: none;
    border: none;
}

.context-menu-box {
    background-color: rgb(14, 14, 18);
    background-image: none;
    opacity: 1.0;
    color: #e8e8e8;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 14px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.65);
    padding: 4px;
    animation: context-menu-pop 160ms cubic-bezier(0.16, 1, 0.3, 1);
}

button.context-menu-item {
    background-image: none;
    background-color: transparent;
    border: none;
    outline: none;
    box-shadow: none;
    border-radius: 8px;
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 500;
    min-height: 28px;
    color: rgba(255, 255, 255, 0.90);
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
    color: rgba(255, 255, 255, 0.90);
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
    opacity: 0.50;
    margin-left: 12px;
    font-feature-settings: "tnum";
    transition: opacity 140ms ease;
}

button.context-menu-item:hover .shortcut-label {
    opacity: 0.85;
}

.context-menu-separator,
.menu-sep {
    margin-top: 4px;
    margin-bottom: 4px;
    margin-left: 6px;
    margin-right: 6px;
    min-height: 1px;
    background-color: rgba(255, 255, 255, 0.10);
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

/* =============================================================================
   Popovers & Tooltip Popover Overlay (Matching babydra-core)
   ============================================================================= */
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
    background-color: rgb(13, 13, 17);
    background-image: none;
    opacity: 1.0;
    color: #ffffff;
    border: 1px solid rgba(255, 255, 255, 0.10);
    border-radius: 18px;
    padding: 14px 18px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.65);
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
    color: #ffffff;
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
    color: rgba(255, 255, 255, 0.50);
}

.status-popover-val,
.tooltip-popover-val {
    font-size: 12px;
    font-weight: 700;
    color: #ffffff;
    font-feature-settings: "tnum";
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

/* Scrollbars */
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
    background-color: rgba(255, 255, 255, 0.20);
    transition: background-color 0.2s ease, border-width 0.2s ease;
}

scrollbar slider:hover,
scrollbar slider:hover:active {
    border-width: 2px;
    background-color: rgba(255, 255, 255, 0.35);
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
