# happyterminals

> Terminal art should feel like magic, not plumbing.

[![CI](https://github.com/lynxnathan/happyterminals/actions/workflows/ci.yml/badge.svg)](https://github.com/lynxnathan/happyterminals/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-1.86%2B-orange.svg)](rust-toolchain.toml)

A declarative, reactive terminal scene manager with composable visual effects
and ASCII 3D rendering. Describe a scene with signals, cameras, and layers --
the framework handles projection, compositing, and ANSI output. Pure text,
every terminal, everywhere.

## Spinning Cube Demo

<!-- TODO: Replace with animated GIF or asciinema recording once captured -->
<!-- asciinema rec / termsvg / vhs recording goes here -->

*A signal-rotated, effect-enhanced ASCII cube rendered entirely in your terminal.*

```
cargo run --example spinning-cube
```

The spinning cube exercises the full stack end-to-end: reactive signals drive
rotation, the 3D rasterizer projects and shades, the effect pipeline composites,
and the ratatui backend paints pure ANSI text to any terminal.

## Hello World

A complete scene in under 20 lines:

```rust
use happyterminals::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (result, _owner) = create_root(|| {
        let rotation = Signal::new(0.0_f32);
        let r = rotation.clone();
        let scene = scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| l.cube().rotation(&rotation))
            .build()?;
        Ok((scene, r))
    });
    let (scene, rotation) = result?;
    run_scene(scene, FrameSpec::default(), |dt, _| {
        rotation.set(rotation.untracked() + dt.as_secs_f32());
    }).await
}
```

## Features

- **Fine-grained reactive signals** -- SolidJS-style `Signal`, `Memo`, `Effect` with surgical cell-level redraws (no VDOM diffing)
- **3D ASCII rendering** -- perspective projection, z-buffer rasterization, shading ramp, orbit camera
- **tachyonfx effects pipeline** -- 50+ composable visual effects via the official ratatui effects library
- **Scene graph DSL** -- declarative builder API inspired by react-three-fiber: cameras, layers, meshes, signals as props
- **Cross-terminal output** -- pure text + ANSI escapes; works on Windows Terminal, macOS Terminal.app, iTerm2, Kitty, tmux, screen, and SSH sessions

## Installation

```bash
cargo add happyterminals
```

Requires Rust 1.86+ (pinned via `rust-toolchain.toml`; `rustup` auto-installs).

The async runtime is tokio (current-thread flavor):

```bash
cargo add tokio --features rt,macros
```

## Crate Structure

| Crate | Role |
|-------|------|
| [`happyterminals`](crates/happyterminals/) | Meta crate -- curated public surface (`use happyterminals::prelude::*`) |
| [`happyterminals-core`](crates/happyterminals-core/) | Reactive primitives (`Signal`, `Memo`, `Effect`) and `Grid` buffer |
| [`happyterminals-renderer`](crates/happyterminals-renderer/) | 3D projection, z-buffer rasterization, shading |
| [`happyterminals-pipeline`](crates/happyterminals-pipeline/) | Effect trait, `Pipeline` executor, tachyonfx adapter |
| [`happyterminals-scene`](crates/happyterminals-scene/) | Scene IR, scene graph, transition manager |
| [`happyterminals-dsl`](crates/happyterminals-dsl/) | Rust builder API (`scene()`, `layer()`, `camera()`) |
| [`happyterminals-backend-ratatui`](crates/happyterminals-backend-ratatui/) | Event loop, `TerminalGuard` RAII, ratatui bridge |

## Development

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md). Contributions are welcome and
will be dual-licensed under the project's terms.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](./LICENSE-MIT) or <https://opensource.org/license/mit>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
