# ADR 0003 — Clean Architecture for `window-manager`

- Status: accepted
- Date: 2026-11-23
- Supersedes: [0001 — Layered module architecture](0001-layering.md)

## Context

The previous refactor produced a layered module graph with cycles between
`compositor`, `input` and `shell`: the god object `BabydraState` had inherent
`impl` blocks spread across those modules, so each layer imported the others.
The layering matrix lived in `xtask/architecture.toml` and was checked in CI by
`cargo xtask check`. That arrangement described the *current* imports rather
than an enforceable architectural boundary, and the enforcement tooling added
maintenance cost without eliminating the cycles.

## Decision

Restructure `crates/window-manager/src/` into Clean Architecture layers with a
strict inward dependency rule:

```text
composition → infrastructure → adapters → application → domain
```

- **domain**: entities and pure rules (`geometry`, `titlebar`, `window`,
  `shortcuts`, `keysyms`). No config files, processes, protocol handles or UI
  toolkits. Geometry is defined locally as plain value types; the window
  metadata cache lives in `adapters/window`.
- **application**: use cases (`shortcuts`, `gestures`, `windows`, `clipboard`,
  `switcher`) plus ports. Config loading, command launching and gesture-policy
  snapshots are injected by the composition root; effectful ports are traits
  that can be replaced by fakes in tests.
- **adapters**: Wayland protocol handlers, input translation, rendering and
  shell overlays. They implement the boundary between the outside world and
  the use cases.
- **infrastructure**: Smithay backends, GTK/platform helpers and the process
  launcher — the frameworks and drivers that satisfy application ports.
- **composition**: the entry point that wires concrete adapters into use cases.

`cargo xtask` and its baseline/matrix files are removed. CI was restored by
[ADR 0004](0004-architecture-gate.md), which adds an automated architecture
fitness test alongside formatting, lint, test and dependency checks.

## Consequences

- `domain` and `application` are independent of Smithay's runtime, GTK and the
  filesystem, so their rules can be unit-tested in isolation. Unit tests are
  inline `#[cfg(test)]` modules next to the code; integration and architecture
  tests live in `tests/`.
- Input configuration, gesture policy and process launching enter the core only
  through ports, making those effects explicit and replaceable.
- Large areas stay split into sub-folders by responsibility; code was not merged
  into large files for the sake of fewer paths.
- The previous cross-layer cycles are no longer part of the intended design;
  new code must follow the inward dependency rule.
- An automated architecture gate in CI keeps imports pointing inward and the
  core framework-free; see [ADR 0004](0004-architecture-gate.md).

## Current status (updated 2026-11-23)

The folder reorganisation in this ADR was completed, but the intended
boundaries were not fully achieved. Follow-up inspection found three remaining
adapter cycles and a `BabydraState` god object; the metadata cache was still
inside `domain`, and `application` still depended on Smithay geometry. Those
statements in the decision and consequence sections therefore describe the
goal, not the state of the tree.

The gap has since been closed in stages: the metadata cache moved to
`adapters/window`, cursor mapping moved out of `domain`, switcher state moved to
`adapters/desktop`, input configuration moved to `infrastructure/config`,
shortcut actions became a typed enum, the shared frame loop moved to
`infrastructure/backends/render_loop`, input now reaches desktop actions only
through the `InputContext` seam, the shortcut service is injected instead of
living in a global `OnceLock`, and `domain`/`application` use framework-free
geometry value types. The architecture gate (ADR 0004) pins the remaining
one-way adapter edges as a shrinking baseline and enforces core purity.
The remaining work is the god-state decomposition and the last effectful ports;
progress is tracked in [report.md](../../report.md).
