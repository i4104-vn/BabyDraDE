# ADR 0001 — Layered module architecture

- Status: accepted
- Date: 2026-11-22

## Context

`crates/window-manager` had two dependency cycles: `compositor` ↔ `rendering`
and `compositor` ↔ `shell` (then named `extensions`). A single god object
held protocol, input, overlay, blur and config state.

## Decision

Adopt one-way layers:

```text
app → backends / shell / input / rendering / compositor / platform / keymap
                     ↓
                   model
```

Rules are encoded in `xtask/architecture.toml` and checked by
`cargo xtask check`. `model` is a leaf that only depends on external libraries.
The baseline in `xtask/baseline.toml` is now empty; init-only I/O exceptions are
listed explicitly in `hot_path_allow` with their rationale.

## Consequences

- New cross-layer imports fail CI.
- `model` hosts shared plain data (`TitlebarLayout`, `WindowMetadata`,
  `WindowVisualState`) so render and input paths do not import each other.
- Remaining debt is visible and shrinking rather than implicit.
