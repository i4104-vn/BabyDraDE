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

- **domain**: entities and pure rules (`titlebar`, `window`, `shortcuts`,
  `cursor`, `keysyms`). No config files, processes, protocol handles or UI
  toolkits; it owns its own metadata cache and keysym table.
- **application**: use cases (`shortcuts`, `gestures`, `windows`, `clipboard`,
  `switcher`) plus ports. Config loading, command launching and gesture-policy
  snapshots are injected by the composition root as plain function ports.
- **adapters**: Wayland protocol handlers, input translation, rendering and
  shell overlays. They implement the boundary between the outside world and
  the use cases.
- **infrastructure**: Smithay backends, GTK/platform helpers and the process
  launcher — the frameworks and drivers that satisfy application ports.
- **composition**: the entry point that wires concrete adapters into use cases.

`cargo xtask` and its baseline/matrix files are removed; CI is removed. Module
visibility and code review now maintain the boundaries.

## Consequences

- `domain` and `application` are independent of Smithay's runtime, GTK and the
  filesystem, so their rules can be unit-tested in isolation; tests live in
  `src/tests/`, mirroring the source tree.
- Input configuration, gesture policy and process launching enter the core only
  through ports, making those effects explicit and replaceable.
- Large areas stay split into sub-folders by responsibility; code was not merged
  into large files for the sake of fewer paths.
- The previous cross-layer cycles are no longer part of the intended design;
  new code must follow the inward dependency rule.
- There is no automated architecture gate; contributors are responsible for
  keeping imports pointing inward.
