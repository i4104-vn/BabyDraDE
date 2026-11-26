# File chooser and `xdg-open`

BabyDraDE owns its native file picker and its `xdg-open` replacement. There are
no standalone helper tools or extra CLI subcommands: the compositor binary
handles everything through the `xdg-open` symlink.

## Ownership

| Crate | Path | Responsibility |
| --- | --- | --- |
| `common` | `libs/common/src/filedialog/` | Serializable data: `FileDialogMode`, `FileDialogOptions`, `FileFilter`, `FileDialogResult`, `QuickAccessPlace` |
| `services` | `libs/services/src/system/filedialog/` | `open` (launch targets), `filter` (glob/MIME), `places` (sidebar bookmarks) |
| `ui-kit` | `libs/ui-kit/src/components/modals/file_chooser/` | GTK4 native chooser (places, breadcrumbs, search, filters, multi-select) |
| `window-manager` | `crates/window-manager/src/modules/shell/filedialog.rs` | Dispatches `xdg-open` calls |

The chooser uses the existing `.file-chooser-*` and `.modern-dialog-*` styles, so
it follows the active light/dark theme and locale.

## Setup

`install.sh` installs the compositor binary and replaces `xdg-open` with it in
every installed binary directory. No manual symlink is needed.

## Behavior

- **With a target:** each path or URL is opened via GIO, falling back to BabyDra
  apps (`babydra-explore`, `babydra-notepad`, `babydra-preview`), `xdg-mime`
  defaults and installed desktop entries. The process exits non-zero if any
  target fails.
- **With no target:** the native GTK chooser opens; the selected path is then
  opened as above.
