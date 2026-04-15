# Contributing to happyterminals

Thanks for considering a contribution! happyterminals is a dual-licensed
(MIT OR Apache-2.0) Rust workspace. This document covers how to get set up,
how the project is organized, and how contributions are licensed.

## Prerequisites

- Rust 1.86 or newer (`rust-toolchain.toml` pins 1.86; `rustup` auto-installs).
- `cargo-machete` for unused-dep detection: `cargo install cargo-machete`.
- `ripgrep` (for the doc-lint script).
- (Optional) `cargo-insta` for snapshot test review: `cargo install cargo-insta`.

## Building and testing

```bash
# Full workspace build
cargo build --workspace

# Tests
cargo test --workspace --all-features

# Lints
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Docs
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Hygiene
cargo tree --workspace --duplicates --edges normal
cargo machete
bash scripts/doc-lint.sh
```

CI (`.github/workflows/ci.yml`) runs every check on each push and PR;
everything above must pass before merge.

## Workspace layout

See the table in the top-level README. In brief: `-core` defines foundational
types, everything else builds on top. `happyterminals-py` (Milestone 4) is the
only crate allowed to depend on `pyo3` — never add `pyo3` to any other member.

## Forbidden strings

The doc-lint CI step fails on these terms outside of
`docs/decisions/stack-rationale.md` (and vendored reference copies):

- `tui-vfx` (use `tachyonfx`)
- `Haskell bindings` (descoped)
- `pyo3-asyncio` (use `pyo3-async-runtimes`)
- `cgmath` (use `glam`)
- `tui-rs` (use `ratatui`)

If you genuinely need to mention one of these in prose (comparing to a prior
art, explaining why we avoid it), add it to `docs/decisions/stack-rationale.md`.

## Commit style

- Keep commits focused. One logical change per commit.
- First line ≤ 72 characters, imperative mood (e.g., `Add TerminalGuard panic hook`).
- Reference phase/requirement IDs where applicable (e.g., `HYG-05: add LICENSE files`).

## License

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as

    MIT OR Apache-2.0

without any additional terms or conditions.
