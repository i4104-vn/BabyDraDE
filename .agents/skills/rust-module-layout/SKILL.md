---
name: rust-module-layout
description: >-
  Restructure a Rust crate's src/ tree into a strict, topic-grouped module layout:
  file/subfolder caps, eliminating single-file folders, visibility, compatibility
  re-exports, and splitting oversized files. Use this whenever the user asks to
  regroup, tidy, simplify or reorganize folders/modules in a Rust crate, or when a
  crate has folders with too many files, chains of single subfolders, or lone
  one-file modules. Pairs with rust-best-practices (§9 modules, §4 visibility).
---

# Rust module layout (strict)

Use this when reorganizing a crate's `src/` tree. Goal: **gọn gàng, dễ quản lý, dễ mở rộng** without changing behavior.

## 1. The hard rules

For every directory under `src/`, except `src/` itself:

1. **At most 6 `.rs` files** directly in a folder.
2. **If a folder has subfolders, it must have at least 2 subfolders.** A lone subfolder is a violation.
3. **If a folder has no subfolders, it must have at least 2 `.rs` files.** A folder with one file is a violation.
4. **A folder with a single `.rs` file and no subfolders must be eliminated** by moving that file to the parent as `name.rs` (`git mv x/mod.rs x.rs`). If the parent already has `x.rs` plus `x/`, merge the folder's contents into `x.rs`.
5. **Root `src/` is exempt** — it may hold single files (`lib.rs`, `main.rs`, `auth.rs`, ...).
6. **Asset-only folders are exempt** from rule 2 (e.g. `i18n/locales/` containing only JSON).

Additional quality rules:

- **Group by topic, not by kind.** Name leaf modules after their primary item (`router/`, `outputs.rs`, `auto.rs`). Avoid `utils/`, `helpers/`, `models/` as a dumping ground when a topic name is clearer.
- **Non-leaf `mod.rs` contains only `mod` declarations and `pub use` re-exports** — no logic (rust-best-practices §9).
- **Re-export individually, never with a glob** (§9). Keep framework preludes (`gtk4::prelude::*`) — they are documented exceptions.
- **Private by default; widen only as needed**: private → `pub(super)` → `pub(crate)` → `pub` (§4).

## 2. Fixing a single-subfolder chain

When `foo/` has `mod.rs` + one subfolder `bar/`, pick one:

- **Flatten**: move `bar/*.rs` into `foo/` and absorb `bar/mod.rs` into `foo/mod.rs`. Valid only if the result has 2–6 files and 0 subfolders.
- **Add a second topic subfolder**: move a coherent pair of files into `foo/baz/`, create sibling `foo/baz.rs` declaring `pub mod ...`, and re-export from `foo/mod.rs` so old paths still resolve.

Names that worked well: `appearance/`, `input/`, `session/`, `device/`, `network/`, `sections/`, `api/`, `motion/`, `buttons/`, `controls/`, `browse/`, `outputs/`, `auto/`.

## 3. Fixing a folder with >6 files

Split by topic into ≥2 subfolders, each with 2–6 files. Keep `mod.rs` as the re-export hub:

```rust
// modules/mod.rs — non-leaf: declarations + re-exports only
mod device;
mod network;

pub use device::{backlight, clean, storage, volume};
pub use network::{bluetooth, vpn, wifi};
```

A sibling `device.rs` / `network.rs` declares the leaf modules, so the topic folders need no `mod.rs` of their own — this keeps file counts low.

## 4. Compatibility without aliases

Moving a file must not break callers. Preserve the old path with a re-export from the parent module:

```rust
// filedialog/mod.rs
mod browse;
pub use browse::{filter, places};   // old paths services::filedialog::filter::* still work
```

If an old path pointed into a folder that no longer exists, add an **inline compatibility module** (no new file, so folder rules still hold):

```rust
// files/mod.rs — callers use services::files::index::*
pub mod index {
    pub use super::{scan_home, FileEntry, MAX_DEPTH, MAX_ENTRIES, SKIP_DIRS};
}
```

Do this only for paths with real external consumers — verify with a workspace-wide grep first.

## 5. Test-only imports under `-D warnings`

A re-export used only by `#[cfg(test)]` tests is `unused_imports` in a normal build. Gate it:

```rust
pub(crate) use api::error;              // used by production code
#[cfg(test)]
pub(crate) use api::snapshot;           // used only by tests
```

Better still: point test imports directly at the new module (`use super::ssid::{...}`) instead of adding a re-export.

## 6. Splitting oversized files

Target files under ~300 lines (be pragmatic). Split by responsibility (state/protocol/config/auto/tests), not by line count alone:

- `outputs.rs` — protocol state machine + Dispatch impls
- `auto.rs` — background loop / session
- `config.rs` — discovery, parsing, validation
- `tests.rs` — the `#[cfg(test)] mod tests` block

Keep moved code **verbatim**; change only visibility and module paths.

## 7. Verification script

Run this from `src/` and require no output:

```bash
bad=0
for d in $(find . -type d ! -path . | sort); do
  f=$(find "$d" -maxdepth 1 -name '*.rs' -type f | wc -l)
  s=$(find "$d" -maxdepth 1 -type d ! -path "$d" | wc -l)
  msg=""
  [ "$f" -gt 6 ] && msg="$msg TOO_MANY($f)"
  [ "$s" -ge 1 ] && [ "$s" -lt 2 ] && msg="$msg ONE_SUBDIR($s)"
  [ "$s" -eq 0 ] && [ "$f" -lt 2 ] && msg="$msg SINGLE_FILE($f)"
  if [ -n "$msg" ]; then echo "$d rs=$f sub=$s$msg"; bad=1; fi
done
[ "$bad" -eq 0 ] && echo ALL_COMPLIANT
```

Also require: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test` (same test count as baseline), `cargo doc --no-deps` (no warnings), `grep -rn 'pub use .*::\*'` → 0, and `cargo check --workspace --all-targets` (downstream not broken).

## 8. Workflow

Plan the **target tree** on paper before touching files. Use `git mv` for every move. Make one commit per logical group so a bad stage can be reverted. See `rust-refactor-workflow`.
