---
name: rust-refactor-workflow
description: >-
  Orchestrate a large Rust refactor safely: baseline snapshot, target-tree plan,
  parallel sub-agents with disjoint file scopes, centralized integration and
  verification gates, atomic commits and rollback. Use when the user asks for a
  big/complete refactor of a crate or directory, especially when they ask to
  split work among several sub-agents. Pairs with rust-module-layout and
  rust-cleanup-checklist.
---

# Large Rust refactor workflow

Use for crate-wide restructuring. The aim is **zero behavior change, zero API breakage**, even when many files move.

## 1. Baseline before anything

Record and, if possible, commit the current state:

```bash
cargo clippy -p <crate> --all-targets --all-features -- -D warnings   # must be clean
cargo test -p <crate>            # record exact count, e.g. 119 passed
cargo doc -p <crate> --no-deps   # record warning count
git status --short
git add -A && git commit -m "checkpoint: before refactor"
```

Also snapshot:
- test names (`cargo test -p <crate> -- --list`) — compare after
- file/subfolder counts per directory (see `rust-module-layout` script)
- public paths used by other crates (workspace-wide `grep -rn 'crate_name::'`)
- file-relative asset paths (`include_str!`, `include_bytes!`, `#[path]`) — moving these files breaks builds

## 2. Plan the target tree first

Write the complete target tree with counts per folder. Every folder must satisfy the rules in `rust-module-layout`. Decide:
- which single-file folders become parent files
- which >6-file folders split into topics
- which single-subfolder chains get a second subfolder
- where compatibility re-exports are needed

Do not start editing until the tree is written down.

## 3. Split agents by DISJOINT scope

Give each sub-agent a distinct top-level path set. Never let two agents own the same file. Typical split for a large crate:

| Agent | Scope |
|---|---|
| A | `app/`, `core/`, `shared/`, crate root |
| B | `protocol/`, `infra/` |
| C | `modules/input/`, `modules/window/` |
| D | `modules/blur/`, `render/`, `auto_rotate/`, `wayland/` |
| E | `modules/shell/` (largest subtree alone) |

Every agent prompt must contain:
- the exact rules (caps, subfolder counts, single-file elimination)
- the exact target tree or violations to fix
- "preserve all public paths via re-exports; no logic/signature/behavior changes"
- "use `git mv`"
- "do NOT run cargo" (avoid concurrent builds and inconsistent results)
- "report files moved/created/deleted, final counts, compatibility paths"

Agents may verify statically (`rustfmt --check`, `git diff --check`, standalone `rustc` scratch files) but leave cargo to the integrator.

## 4. Integrate centrally

Expect small cross-agent breakage. Common fixes:
- **Unused re-exports**: a re-export used only by tests triggers `unused_imports` under `-D warnings` → add `#[cfg(test)]` or point test imports directly at the module.
- **Broken `super::` paths**: a moved file's `super::x` now resolves to the wrong parent → switch to a crate-absolute path or re-export from the new parent.
- **Doc links to private items**: convert `[`x`]` to plain code spans.
- **Glob re-exports**: replace with explicit lists.

## 5. Verification gates (run after integration, before commit)

1. `rust-module-layout` folder-count script → `ALL_COMPLIANT`
2. `cargo fmt -p <crate>`
3. `cargo clippy -p <crate> --all-targets --all-features -- -D warnings` → clean
4. `cargo test -p <crate>` → **same count** as baseline
5. `cargo doc -p <crate> --no-deps` → no warnings
6. `grep -rn 'pub use .*::\*' src` → 0 (framework preludes exempt)
7. `cargo check --workspace --all-targets` → downstream crates still compile
8. `git diff --stat` → mostly renames/additions, no logic changes

Stop at the first failing gate; fix before continuing.

## 6. Commits and rollback

- One checkpoint commit before the refactor.
- One commit per logical stage (or one per crate if agents' work is interdependent).
- If a stage fails verification: `git revert <sha>` or `git reset --hard <previous>`, never stack the next stage on top.
- Report pre-existing failures separately (e.g. clippy errors in other crates) and do not silently fix out-of-scope code.

## 7. What to tell the user

Give a short table: scope → changes → verification result. State the exact commits, the test count, and any deliberate non-changes (quirks kept, external lints untouched).
