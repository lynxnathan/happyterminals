# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Pre-1.0 versioning: minor bumps (0.X.0) may contain breaking changes per Cargo's
semver interpretation; patch bumps (0.0.X) are fixes only.

## [Unreleased]

### Added
- (Phase 1.0+ additions accumulate here.)

### Changed

### Removed

## [0.0.0] — 2026-04-14

Initial workspace scaffolding (Phase 0: Workspace Hygiene & Foundation).

### Added
- Dual-license files at repo root: `LICENSE-MIT` and `LICENSE-APACHE`, with every
  crate's `Cargo.toml` carrying SPDX `MIT OR Apache-2.0`.
- `[workspace.dependencies]` block pinning every shared crate version
  (ratatui 0.30, tachyonfx 0.25, glam 0.32.1, reactive_graph 0.2.13, pyo3 0.28.3,
  and the supporting cast). All member crates inherit via `dep.workspace = true`.
- `rust-toolchain.toml` pinned to Rust 1.86 with `clippy` + `rustfmt` components.
- New workspace members: `happyterminals` (meta), `happyterminals-scene`,
  `happyterminals-dsl`, `happyterminals-backend-ratatui`. Placeholder for
  `happyterminals-py` (activated in Milestone 4).
- `.gitattributes` marking `vendor/_reference/**` as `linguist-vendored=true`.
- `scripts/doc-lint.sh` that fails CI on forbidden strings outside allowlist.
- `.github/workflows/ci.yml`: fmt, clippy `-D warnings`, test (Rust 1.86 + stable),
  `cargo doc -D warnings`, duplicate-dep scan, `cargo-machete`, doc-lint.
- `CONTRIBUTING.md` with Apache-2.0 §5 contribution clause.
- `docs/decisions/stack-rationale.md` as the sole allowlist for forbidden terms.

### Changed
- Renamed crate `happyterminals-compositor` → `happyterminals-pipeline` (the
  term `Pipeline` is used throughout the roadmap and API design; `compositor`
  was a legacy name).
- Rewrote `README.md` and `project.md` to reflect the current stack (no stale
  `tui-vfx` claims; no Haskell-bindings references).
- Vendored reference copies moved from `vendor/<name>/` to `vendor/_reference/<name>/`
  with `STAMP.txt` provenance files; never referenced via `path =` dependencies.
- Workspace `resolver` upgraded from `"2"` to `"3"` to match `edition = "2024"`.

### Removed
- Speculative dependencies from stub crates (`-core` had `pyo3`, `tui-vfx`,
  `ratatui`; `-compositor` had `tui-vfx`). Stubs now carry no dependencies
  until real call sites land in Phase 1.0+.

[Unreleased]: https://github.com/lynxnathan/happyterminals/compare/v0.0.0...HEAD
[0.0.0]: https://github.com/lynxnathan/happyterminals/releases/tag/v0.0.0
