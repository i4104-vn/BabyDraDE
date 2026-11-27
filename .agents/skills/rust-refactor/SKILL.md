---
name: rust-refactor
description: >-
  End-to-end playbook for a large Rust refactor: strict module layout (file/subfolder
  caps, no single-file folders), splitting oversized files, parallel sub-agents with
  disjoint scopes, shared-code extraction into libs/common and libs/ui-kit/components,
  comment/rustdoc cleanup, and mandatory verification gates. Use whenever the user
  asks to refactor, regroup, tidy, restructure or reorganize a Rust crate or src/
  tree, especially when they ask to split work among sub-agents. Supersedes the
  separate rust-module-layout, rust-refactor-workflow, rust-cleanup-checklist and
  rust-shared-extraction skills. Pairs with rust-best-practices and rust-review.
---

# Rust refactor playbook

One skill, **eleven ordered steps**. Run them in order; do not skip a gate. Goal: tidy, maintainable structure with **zero behavior or public-API change**.

---

## Step 1 — Baseline & checkpoint

Record before touching code:

```bash
cargo clippy -p <crate> --all-targets --all-features -- -D warnings
cargo test -p <crate>                  # record exact count, e.g. 111 passed
cargo doc -p <crate> --no-deps         # record warning count
git status --short
git add -A && git commit -m "checkpoint: before refactor"
```

Also snapshot:
- test names (`cargo test -p <crate> -- --list`) for after-comparison
- folder counts via the script in Step 4
- workspace-wide uses of the crate's public paths (`grep -rn '<crate>::'`)
- file-relative assets: `include_str!`, `include_bytes!`, `#[path]` — moving those files breaks builds

---

## Step 2 — Plan the target tree

Write the complete target tree on paper **before editing**. Decide:
- which single-file folders become `name.rs` at the parent
- which >6-file folders split into topics
- which single-subfolder chains get a second subfolder
- which oversized files split by responsibility
- which compatibility re-exports are needed

Do not edit until the tree is written down.

---

## Step 3 — Split work among sub-agents (disjoint scopes)

Give each agent a distinct top-level path set; never let two own the same file. Every prompt must include: the rules below, the exact violations/target tree, "preserve public paths via re-exports, no logic/signature/behavior changes", "use `git mv`", and "**do not run cargo**" (central verification only).

Typical split for a large crate: `app/core/shared/root` · `protocol/infra` · `modules/input + window` · `modules/blur + render + wayland` · `modules/shell`.

Agents may verify statically (`rustfmt --check`, `git diff --check`, standalone `rustc` scratch files).

---

## Step 4 — Enforce the strict layout rules

For every folder under `src/` except `src/` itself:

1. **≤6 `.rs` files** directly in a folder.
2. **Subfolders ≥2 if any exist.** A lone subfolder violates.
3. **No subfolders → ≥2 `.rs` files.**
4. **Single-file folder → eliminate** by `git mv x/mod.rs x.rs`. If parent already has `x.rs` + `x/`, merge contents into `x.rs`.
5. **Root `src/` exempt**; asset-only folders (e.g. JSON `locales/`) exempt from rule 2.
6. **Group by topic, not by kind**; leaf named after its primary item; non-leaf `mod.rs` contains only `mod`/`pub use`.
7. **Private by default**, widen private → `pub(super)` → `pub(crate)` → `pub` only when a consumer needs it.

Fix a single-subfolder chain by flattening (if 2–6 files result) or by adding a second topic folder with a sibling declaring file:

```rust
// controller/mod.rs
mod input;                                  // sibling input.rs declares its leaves
pub use input::{keyboard, wiring};          // old paths survive
```

Fix a >6-file folder by splitting into ≥2 topic folders, keeping `mod.rs` as a re-export hub.

Check with:

```bash
cd src && bad=0
for d in $(find . -type d ! -path . | sort); do
  f=$(find "$d" -maxdepth 1 -name '*.rs' -type f | wc -l)
  s=$(find "$d" -maxdepth 1 -type d ! -path "$d" | wc -l)
  msg=""
  [ "$f" -gt 6 ] && msg="$msg TOO_MANY($f)"
  [ "$s" -ge 1 ] && [ "$s" -lt 2 ] && msg="$msg ONE_SUBDIR($s)"
  [ "$s" -eq 0 ] && [ "$f" -lt 2 ] && msg="$msg SINGLE_FILE($f)"
  [ -n "$msg" ] && { echo "$d rs=$f sub=$s$msg"; bad=1; }
done
[ "$bad" -eq 0 ] && echo ALL_COMPLIANT
```

---

## Step 5 — Split oversized files

Target <~300 lines (feature crates may use ~200). Split by **responsibility**, not line count: `mod.rs` (entry), `outputs.rs` (protocol/state), `auto.rs` (background loop), `config.rs` (discovery/parsing), `tests.rs`. Move code **verbatim**; change only visibility and paths. Prefer `git mv original.rs original/mod.rs` so history is preserved.

---

## Step 6 — Integrate & fix cross-agent fallout

Run clippy and expect small breakage. Known fixes:

| Symptom | Fix |
|---|---|
| `private method` E0624 between sibling modules | widen the method to `pub(super)` (invisible outside the parent) |
| `unused_imports` on a re-export used only by tests | add `#[cfg(test)]`, or point test imports directly at the module |
| broken `use super::x` after a move | use a crate-absolute path or re-export from the new parent |
| doc link to private/moved item | plain code span `` `x` `` instead of `[`x`]` |
| glob re-export | explicit list |

Do not silence lints: prefer `#[expect(clippy::name, reason = "...")]` on the narrow item, never `#![deny(warnings)]` in source.

---

## Step 7 — Extract shared code into `libs`

Done **after** the tree is clean — never mix with folder moves.

| Code | Destination | Rule |
|---|---|---|
| Shared enums, constants, value types, protocol models | `libs/common/src/constants/`, `libs/common/src/enums/`, ... | Name after the **concept** (`PowerAction`, `MonitorOrientation`), not the consumer |
| Reusable widgets, panels, popovers, dialogs, render helpers | `libs/ui-kit/src/components/<component>/` | Name after **what the component is** (`file_chooser`, `context_menu`, `slider`), not the service (`tray_widget`, `greeter_panel`) |
| Service implementations, hardware drivers | stay in `libs/services` or feature crate | — |

Procedure:
1. Inventory duplicates: `grep -rnE '^(pub )?(enum|const|struct) ' crates libs --include='*.rs'`. Compare fields, not just names.
2. Pick owner by concept, not current caller.
3. Move the definition; update all call sites. Keep the old path with a `pub use` **re-export** (one source of truth, no duplicated definition).
4. Ensure consuming crates depend on `common`/`ui-kit` in `Cargo.toml`; versions via `[workspace.dependencies]`.
5. `libs` code must be library-grade: derive common traits, `Display`/`Error` for errors, `&str`/`&[T]`/`impl AsRef<_>` params, no `Rc`/`RefCell`/`Arc` in public signatures, `# Errors`/`# Panics` docs.
6. Re-run Step 4 script in both donor and destination crates.

---

## Step 8 — Clean code & comments

- Delete dead imports, no-ops, redundant `return`/derefs, commented-out code, stale TODOs, comments restating the identifier.
- **Keep** `//!`/`///`, `# Errors`/`# Panics`/`# Safety`, `// SAFETY:`, and "why" comments (quirks, workarounds, contracts).
- Fix **inaccurate** docs rather than deleting them.
- Write concise rustdoc: first sentence one line ≈15 words, says what the signature cannot; never restates the name.
  ```rust
  /// Converts a configured orientation into the Wayland protocol transform.  // good
  /// Converts orientation.                                                    // noise
  ```
- Leave suspicious but load-bearing code alone and report it (e.g. deliberate double initializer, required clone for recursion).

---

## Step 9 — Verification gates (run in order, stop on failure)

1. Step 4 folder script → `ALL_COMPLIANT` in every touched crate
2. `cargo fmt` on each touched crate
3. `cargo clippy -p <crate> --all-targets --all-features -- -D warnings` → clean
4. `cargo test -p <crate>` → **same count** as Step 1 baseline
5. `cargo doc -p <crate> --no-deps` → no warnings
6. `grep -rn 'pub use .*::\*' src` → 0 (framework preludes exempt)
7. `cargo check --workspace --all-targets` → downstream crates compile
8. `git diff --stat` → mostly renames/additions, no logic changes

---

## Step 10 — Commit & report

One checkpoint commit before the refactor; one commit per logical stage (or per crate when agents' work is interdependent). On failure: `git revert <sha>` / `git reset --hard <previous>` — never stack the next stage on a broken one.

Report a short table: scope → changes → verification result, exact commits, test count, and deliberate non-changes (quirks kept, out-of-scope lints untouched).

---

## Step 11 — Repeat for the next crate

Return to Step 1 for the next target. Keep each crate's baseline and gates independent so failures stay local.
