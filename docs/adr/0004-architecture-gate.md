# ADR 0004 — Architecture gate in CI

- Status: accepted
- Date: 2026-11-23
- Amends: [0003 — Clean Architecture](0003-clean-architecture.md)

## Context

ADR 0003 removed CI and stated that the previous cross-layer cycles had been
eliminated, leaving boundary enforcement to code review and module visibility.
That claim did not hold: three cycles between `compositor`, `input`, `rendering`
and `shell` remain in the adapter layer. Without an automated gate, the cycles
can silently grow again while the refactor is in progress.

## Decision

Restore a minimal CI workflow running formatting checks, Clippy with warnings
denied, workspace tests, dependency policy checks and an architecture fitness
test. The fitness test in `crates/window-manager/tests/architecture.rs` forbids
`domain` and `application` from importing outer layers and pins the currently
observed cross-area adapter imports as a temporary baseline. The baseline is not
a seal of approval: it must shrink as the refactor breaks the three cycles, and
no new import edge may be added.

## Consequences

- Every push and pull request is checked automatically; new outward imports or
  new cross-area adapter edges fail the build.
- Contributors update the baseline deliberately, and only to remove edges, when
  a cycle is actually broken.
- The remaining debt stays visible in one place until the refactor described in
  [report.md](../../report.md) removes it.
- CI needs occasional toolchain and action maintenance, but all checks are
  reproducible locally with the commands in `docs/testing.md`.
