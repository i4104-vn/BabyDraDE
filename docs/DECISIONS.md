# Architecture Decisions

This file collects the decisions that shaped the current tree. Each decision
records the context, the decision itself and its consequences in one place.

## 1. Layered module architecture (2026-11-22, superseded)

`crates/window-manager` had two dependency cycles: `compositor` ↔ `rendering`
and `compositor` ↔ `shell` (then named `extensions`). A single god object held
protocol, input, overlay, blur and configuration state.

The first step was one-way layers:

```text
app → backends / shell / input / rendering / compositor / platform / keymap
                     ↓
                   model
```

Rules were encoded in `xtask/architecture.toml` and checked by
`cargo xtask check`. `model` was a leaf depending only on external libraries.
The baseline was expected to shrink until empty, with init-only I/O exceptions
listed explicitly.

**Consequences:** new cross-layer imports failed CI; shared plain data moved to
`model`; remaining debt became visible. The `xtask` enforcement was later
removed, and the layering decision was superseded by Clean Architecture below.

## 2. Split configuration across files (2026-11-22, accepted)

`~/.babydra/babydra.conf` contained every subsystem section. Saving one setting
rewrote the whole file, and input hot paths re-read it on every event.

Only central settings remain in `babydra.conf`: `1st`, `[autostart]`, `[power]`,
`[wallpaper]` and `[notification]`. Subsystem settings moved to
`~/.babydra/configs/*.toml`, with `input.toml` grouping mouse, touchpad and
gestures. `services::config::store` provides mtime-cached, atomic per-file
writes. Legacy sections migrate automatically at startup and the target file
always wins.

**Consequences:** smaller files saved independently; unknown keys in the main
file survive saves; input code consumes snapshots instead of reading disk.
Details in [CONFIGURATION.md](CONFIGURATION.md).

## 3. Clean Architecture (2026-11-23, accepted; supersedes decision 1)

The folder reorganisation alone did not establish the intended boundaries:
three adapter cycles remained, `BabydraState` was still a god object, the
metadata cache lived in `domain`, and `application` depended on Smithay
geometry.

The crate now follows a strict inward rule:

```text
composition → infrastructure → adapters → application → domain
```

- `domain` owns pure rules and its own geometry value types. It has no config
  files, processes, protocol handles or toolkits.
- `application` owns use cases and effectful traits. Adapters and the
  composition root inject implementations; nothing lives in global state.
- `adapters` translate protocol, input, rendering and desktop UI.
- `infrastructure` supplies backends, config loaders and the process launcher.
- `composition` wires concrete implementations and owns the runtime lifecycle.

**Consequences:** the inner layers can be tested without Smithay or GTK; unit
tests are inline and integration tests live in `tests/`; the cross-layer cycles
are gone and the adapter graph is one-way. The remaining debt is listed in
[ARCHITECTURE.md](ARCHITECTURE.md#8-known-debt).

## 4. Architecture gate in CI (2026-11-23, accepted)

An earlier version of the project removed CI and relied on review and module
visibility. The cycles returned. The gate was therefore restored as a set of
automated checks:

- formatting, Clippy with warnings denied, workspace tests and `cargo deny`;
- an architecture fitness test that forbids outer imports from the core, keeps
  the core framework-free and free of service locators, bans hidden dead
  imports, verifies a single toplevel commit owner, and pins the adapter import
  graph as a shrinking baseline.

The baseline is not approval: it shrank from seven edges with cycles to six
acyclic edges and must keep shrinking. An edge may only be removed, never added.

**Consequences:** every push and pull request is checked automatically; the
remaining debt stays visible in one file; all checks are reproducible locally
with the commands in [DEVELOPMENT.md](DEVELOPMENT.md).

## 5. Relocate the protocol crate under the window manager (2026-11-26, historical — superseded by 6)

Features and the compositor originally shared a top-level `crates/protocol/`
crate whose `FileTransport` queued envelopes as files in the runtime directory;
focused-window metadata was written separately as a JSON snapshot. File handoff
lost events, left stale state behind and made ordering depend on polling.

At this point the crate lived at `crates/window-manager/src/protocol/` and was
still an independent Cargo crate, so features linked only the small contract
instead of the heavy compositor crate. Delivery moved to a pure `Router` state
machine hosted by the compositor: peers exchange newline-framed JSON over uid-scoped abstract
Unix sockets, requests are correlated by id, presence is published on
`bus/peers`, and topic values can be retained for late subscribers until a TTL
expires. No filesystem queues or runtime files are created.

**Consequences:** focused-window metadata arrives as a retained `window.active`
event and window actions are requests to the `wm` peer. The frame loop no longer
polls for protocol traffic, and feature code uses thin bus helpers while the
router owns all delivery policy.

## 6. Dissolve the protocol crate into feature-owned modules (2026-11-26, accepted)

Keeping a second Cargo crate nested in `crates/window-manager/src/protocol/`
forced a duplicate manifest and blurred ownership: the compositor, the shared
value types and the feature client all lived in one package that every consumer
linked in full.

The standalone crate was removed. Ownership now follows the layer that uses
each part:

- `window_manager::protocol` — the compositor module at
  `crates/window-manager/src/protocol/`: `Router`, `Connection`, `ReadResult`,
  socket helpers and `BusSender`, hosted by socket-driven calloop sources.
- `common::protocol` — the shared wire types and codec (`Request`, `Response`,
  `Event`, `Envelope`, `MessageId`, `ProtocolError`, ...).
- `services::protocol` — the feature-side `Bus` client, so apps link the service
  layer and never the compositor.

**Consequences:** one manifest less, no duplicate manifest to keep in sync, and
each consumer depends only on the protocol surface it actually uses. The router,
socket framing and retained-topic policy are unchanged from decision 5.
