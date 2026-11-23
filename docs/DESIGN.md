# Design System

The visual language is **Modern Obsidian Acrylic & Precision Glassmorphism**.
It is shared by the compositor overlays, the topbar, the desktop and the
supporting GTK applications.

## 1. Principles

1. **Obsidian Deep Matte.** Surfaces use `rgb(14, 14, 18)` or
   `rgb(13, 13, 17)` for strong contrast on OLED and IPS panels. Instead of
   expensive real-time blur, surfaces use near-full opacity
   (`rgba(20, 22, 34, 0.98)`) with a thin translucent border. Layering reads
   clearly while frame pacing stays stable.
2. **Floating Islands and Capsules.** Controls, status groups and Dynamic
   Islands are capsules with `border-radius: 9999px`, floating above the
   wallpaper with generous padding.
3. **No-Arrow Popovers.** Menus and tooltips use `has_arrow(false)`, soft corner
   radii of 14–18 px and a deep shadow (`0 16px 48px rgba(0,0,0,0.50)`).
4. **Tabular Numbers.** Key/value pairs use a muted key and a bright value.
   Clocks and live metrics use tabular numerals so the layout never jitters.

## 2. Colour tokens

| Token | Dark value | Use |
| --- | --- | --- |
| `surface` | `rgb(14, 14, 18)` | Popovers, capsules, topbar, dock |
| `surface-card` | `rgb(24, 25, 31)` | Cards inside popovers |
| `surface-elevated` | `rgba(20, 22, 34, 0.98)` | Modal dialogs and alerts |
| `border` | `rgba(255,255,255,0.12)` | Container and button borders |
| `border-subtle` | `rgba(255,255,255,0.08)` | Dividers |
| `border-highlight` | `rgba(255,255,255,0.24)` | Top highlight of a modal |
| `scrim` | `rgba(0,0,0,0.65)` | Modal backdrop |
| `text-primary` | `rgba(255,255,255,0.95)` | Titles and values |
| `text-secondary` | `rgba(255,255,255,0.60)` | Keys and descriptions |
| `text-muted` | `rgba(255,255,255,0.40)` | Disabled or pending text |
| `hover-fill` | `rgba(255,255,255,0.10)` | Hover on buttons and rows |
| `hover-fill-subtle` | `rgba(255,255,255,0.06)` | Hover on search fields |

Light mode uses `#ffffff` surfaces, `rgba(0,0,0,0.08)` borders,
`rgba(0,0,0,0.60)` secondary text and `rgba(0,0,0,0.05)` hover fill.

Accent themes:

| Theme | Dark | Light |
| --- | --- | --- |
| `babydra-default` | `#3b82f6` | `#3b82f6` |
| `babydra-blue` | `#38bdf8` | `#0284c7` |
| `babydra-green` | `#10b981` | `#047857` |
| `babydra-purple` | `#8b5cf6` | `#6d28d9` |
| `babydra-rose` | `#f43f5e` | `#be123c` |

Status colours: success `#22c55e`, warning `#f59e0b`, danger `#ef4444`,
active indicator `#3b82f6`.

## 3. Typography and icons

```css
* {
    font-family: 'Quicksand', 'Segoe UI Variable Static Text', 'Segoe UI',
                 'Inter', system-ui, sans-serif;
    -gtk-icon-style: symbolic;
}

.panel-clock, .status-popover-val, .sys-monitor-val {
    font-variant-numeric: tabular-nums;
}
```

| Level | Size | Weight | Line height |
| --- | --- | --- | --- |
| Display | 16 px | 800 | 1.2 |
| Heading | 14 px | 700 | 1.3 |
| Body | 13 px | 500–600 | 1.4 |
| Caption | 12 px | 600 | 1.2 |
| Micro | 9–10 px | 700 | 1.0 |

Icons are symbolic so they inherit the contextual colour. Standard sizes are
16 px for menus and indicators, 14 px inside the island, 20–24 px for quick
tiles, 32–42 px for dialog badges and 48 px for desktop icons.

## 4. Radius and shadows

```text
pill   : 9999px       topbar capsules, Dynamic Island, action buttons
lg     : 20–24px      launcher, control centre, dialogs
md     : 14–18px      popovers, context menus, quick tiles
sm     : 8–12px       small buttons, entries, dropdowns
micro  : 4–6px        badges and tags
```

Floating popovers use `0 16px 48px rgba(0,0,0,0.50)`; dark tooltips use
`0 16px 40px rgba(0,0,0,0.65)`; notification cards use
`0 8px 24px rgba(0,0,0,0.25)`. Panels and island notches that sit on the
desktop stay flat with `box-shadow: none`.

## 5. Components

- **Topbar:** 36 px tall, 8 px top margin. Three regions: left (app menu and
  focused app name), centre (Dynamic Island) and right (system monitor,
  indicators, tray, clock). Each group is a flat capsule.
- **Dynamic Island:** shrinks to its minimum when idle. When media, recording
  or pinning is active, it uses a pill with bracket glyphs and four animated
  visualiser bars in the current accent colour.
- **Popovers:** no arrow, 18 px radius, key column at 50% opacity and value
  column at full white with tabular numbers.
- **Context menus:** appear at the pointer, 190–200 px wide, 16 px radius,
  8 px × 12 px rows with 16 px icons and 8 px gaps.
- **Toggle switch:** 46 × 24 px, drawn with Cairo, animated with
  `ease_out_cubic`: `progress(t) = 1 - (1 - t)^3`.
- **Dialogs:** 65% black scrim, 20 px radius, 24 px padding, 98% opaque elevated
  surface, 12 px squircle badge and a pill-shaped primary button.

## 6. Motion

Micro-interactions use `transition: all 200ms ease`. Island morphing uses
`transition: all 400ms cubic-bezier(0.16, 1, 0.3, 1)`. During startup the
`.startup-animating` class disables transitions to avoid a flash of unstyled
content. Frame-driven compositor motion is described in
[ANIMATIONS.md](ANIMATIONS.md).
