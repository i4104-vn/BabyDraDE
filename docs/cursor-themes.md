# Cursor themes

The compositor draws the pointer itself with its own renderer (no GTK/GDK cursor
code). Every cursor role — arrow, text/I-beam, busy, progress, all resize
directions, grab, links, zoom, etc. — is loaded from the theme and cached as a
render buffer. Cursor images come from themes installed under
`~/.babydra/cursor/`.

## Installing a theme

The directory layout mirrors a standard GTK/Xcursor theme:

```text
~/.babydra/cursor/
└── MyTheme/
    ├── index.theme          # optional, standard [Icon Theme] file
    └── cursors/
        ├── left_ptr         # Xcursor image (no extension), preferred
        ├── ibeam            # text cursor (alias of "text")
        ├── watch            # busy cursor (alias of "wait")
        ├── nwse-resize      # diagonal resize cursor
        └── text.png         # PNG images are supported too
```

`~/.babydra/cursor/` is created automatically on first launch. The compositor
looks there first, then in the usual XDG icon locations
(`~/.icons`, `~/.local/share/icons`, `/usr/share/icons`,
`/usr/local/share/icons`), so system-installed themes work too.

For each role, the W3C cursor name (e.g. `text`, `wait`, `nwse-resize`) is tried
first, followed by legacy Xcursor aliases (e.g. `ibeam`, `watch`,
`bd_double_arrow`). Follow `Inherits=` in `index.theme` to reuse images from a
parent theme. Roles missing everywhere fall back to the built-in arrow, so
partial themes still work.

### Xcursor images

Standard binary cursor files (the format used by GTK themes) are parsed
directly, including their built-in hotspot and the size closest to the
configured one. No external library is used. Animated cursors use their first
frame.

### PNG images

A PNG named after any supported cursor name (e.g. `text.png`) can be used
instead. Since PNG has no hotspot metadata, set it in the config (default
`0, 0`, i.e. the top-left corner, which suits most arrow images). The hotspot
setting applies to all PNG cursors in the theme.

## How the active shape is chosen

- Clients request shapes through `wp_cursor_shape_v1`, which the compositor
  advertises. Setting `text` over an input field, `wait` while loading, etc.
  switches the rendered cursor immediately.
- While a window is being moved or resized by the compositor, a grab/directional
  resize cursor overrides the client's last request.
- Client-supplied cursor surfaces (legacy `wl_pointer.set_cursor`) are not
  rendered; the last themed shape stays visible.

## Selecting a theme

Edit `~/.babydra/babydra.conf`:

```toml
[cursor]
theme = "MyTheme"
size = 24          # preferred size for Xcursor themes
hotspot_x = 0      # PNG only
hotspot_y = 0      # PNG only
```

The setting is read on compositor startup; restart BabyDraDE after changing it.
