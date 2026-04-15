# Stack Rationale — Why Not X

This file is the sole allowlist for the doc-lint CI step. Historical pivots
and anti-recommendations are documented here so contributors understand *why*
the chosen stack is the chosen stack.

## Why not tui-vfx (chose tachyonfx)

tui-vfx was a promising pre-1.0 project (8★, ~5 weeks old at happyterminals'
start) but had no community adoption. tachyonfx (1,182★, 0.25.0, maintained
under the ratatui org with 50+ effects, DSL, WASM editor) is the durable
foundation. See `.eclusa/research/STACK.md` §1.3.

## Why not pyo3-asyncio (chose pyo3-async-runtimes)

`pyo3-asyncio` last released 2023-11-11 and does not support pyo3 ≥ 0.21.
The PyO3 org forked it as `pyo3-async-runtimes`; that's where development
happens in 2026. Any tutorial older than late-2024 recommending the old
crate is stale.

## Why not cgmath (chose glam)

cgmath last release was 2021-01-03; effectively unmaintained. glam (0.32.1,
2026-03-06) is the 2026-standard graphics-math crate — used by Bevy, wgpu
examples, rend3, and most of the modern Rust graphics ecosystem.

## Why not tui-rs (chose ratatui)

tui-rs was deprecated in 2023 when its maintainer stepped down; ratatui is
the community-maintained continuation and the de-facto standard in 2026.

## Why not Haskell bindings (chose Python-only)

Per user decision 2026-04-14: removed from scope. Python covers the creative
scripting layer; Eclusa and other consumers use either the Python layer or
the Rust crate directly. Haskell FFI was retained as a speculative option in
the original manifesto and has been fully scrubbed.
