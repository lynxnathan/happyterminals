# happyterminals

> Terminal art should feel like magic, not plumbing.

[![CI](https://github.com/lynxnathan/happyterminals/actions/workflows/ci.yml/badge.svg)](https://github.com/lynxnathan/happyterminals/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-1.86%2B-orange.svg)](rust-toolchain.toml)

A declarative, reactive terminal scene manager with composable visual effects
and ASCII 3D rendering. Pure text output — runs on every terminal ever made.

**Status:** Pre-alpha. Phase 0 (workspace hygiene) is in progress.
The first milestone is a signal-driven spinning cube demo.

## Stack

- **[ratatui](https://ratatui.rs/)** for terminal I/O (via crossterm)
- **[tachyonfx](https://github.com/ratatui/tachyonfx)** for the effects library
- **[reactive_graph](https://crates.io/crates/reactive_graph)** (Leptos's reactive core) for fine-grained signals
- **[glam](https://crates.io/crates/glam)** for 3D math
- Fresh ASCII rasterizer (not a fork of any existing renderer)

See [`project.md`](./project.md) for the design manifesto
and [`docs/decisions/stack-rationale.md`](./docs/decisions/stack-rationale.md)
for "why not X" decisions.

## Crates

| Crate | Role |
|-------|------|
| `happyterminals-core` | Reactive primitives (Signal, Memo, Effect) and Grid buffer |
| `happyterminals-renderer` | 3D projection, z-buffer rasterization, mesh loading |
| `happyterminals-pipeline` | Effect trait, Pipeline executor, tachyonfx adapter |
| `happyterminals-scene` | Scene IR and scene graph |
| `happyterminals-dsl` | Rust builder API + JSON recipe loader |
| `happyterminals-backend-ratatui` | Event loop + panic-safe TerminalGuard |
| `happyterminals` | Meta crate — curated public surface (`use happyterminals::prelude::*`) |
| `happyterminals-py` | Python bindings (Milestone 4, not yet activated) |

## Quick Start

> This is the target shape; the spinning-cube example ships with Milestone 1.

```rust
use happyterminals::prelude::*;

fn main() -> Result<()> {
    let rotation = signal(0.0_f32);
    let scene = scene()
        .layer(|l| l.cube().rotation(&rotation))
        .effect(fx::vignette(0.3))
        .build()?;

    run(scene, FrameSpec::fps(30))
}
```

## Development

Requires Rust 1.86+ (pinned via `rust-toolchain.toml`; `rustup` auto-installs).

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

Docs, duplicate-dep scan, and unused-dep scan:

```bash
cargo doc --workspace --no-deps
cargo tree --workspace --duplicates
cargo install cargo-machete && cargo machete
```

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md). Contributions are welcome and
will be dual-licensed under the project's terms — see the License section
below.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](./LICENSE-MIT) or https://opensource.org/license/mit)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
