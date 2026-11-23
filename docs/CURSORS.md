# Cursor Themes

The compositor draws the pointer itself with its own renderer; it does not use
GTK or GDK cursor code. Every role — arrow, text I-beam, busy, progress, resize
directions, grab, links, zoom — is loaded from the theme and cached as a render
buffer. Cursor images come from themes installed under `~/.babydra/cursor/`.

## 1. Installing a theme

The directory layout mirrors a standard GTK/Xcursor theme:

```text
~/.babydra/cursor/
└── MyTheme/
    ├── index.theme          # optional, standard [Icon Theme] file
    └── cursors/
        ├── left_ptr         # Xcursor image, no extension, preferred
        ├── ibeam            # text cursor, alias of "text"
        ├── watch            # busy cursor, alias of "wait"
        ├── nwse-resize      # diagonal resize cursor
        └── text.png         # PNG images are supported too
```

`~/.babydra/cursor/` is created on first launch. The compositor looks there
first, then in the usual XDG icon locations (`~/.icons`,
`~/.local/share/icons`, `/usr/share/icons`, `/usr/local/share/icons`), so
system-installed themes work too.

For each role the W3C cursor name is tried first, followed by legacy Xcursor
aliases. `Inherits=` in `index.theme` reuses images from a parent theme. Roles
missing everywhere fall back to the built-in arrow, so partial themes work.

Xcursor files are parsed directly, including their hotspot and the size closest
to the configured one. Animated cursors use their first frame.

A PNG named after a supported cursor name can be used instead. PNG has no
hotspot metadata, so the hotspot is set in the configuration and applies to all
PNG cursors in the theme.

## 2. How the active shape is chosen

- Clients request shapes through `wp_cursor_shape_v1`, which the compositor
  advertises. Hovering an input field with `text` or loading with `wait`
  switches the rendered cursor immediately.
- During a compositor move or resize, a grab or directional resize cursor
  overrides the client's last request.
- Legacy `wl_pointer.set_cursor` surfaces are not rendered; the last themed
  shape remains visible.

## 3. Selecting a theme

Edit `~/.babydra/configs/cursor.toml`:

```toml
[cursor]
theme = "MyTheme"
size = 24          # preferred size for Xcursor themes
hotspot_x = 0      # PNG only
hotspot_y = 0      # PNG only
```

The setting is read at compositor startup; restart BabyDraDE after changing it.
See [CONFIGURATION.md](CONFIGURATION.md) for the full configuration layout.
