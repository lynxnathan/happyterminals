# Deferred Items — Phase 03.4 examples-library

Out-of-scope issues discovered during plan execution but not fixed because they
fall outside the plan's SCOPE BOUNDARY (pre-existing issues in files/crates
unrelated to the current task's changes).

## From Plan 03.4-01 (Prelude Extension)

### Pre-existing clippy errors in crates/happyterminals/examples/model-viewer/main.rs

Discovered during `cargo clippy -p happyterminals --all-targets -- -D warnings`
while verifying the prelude-extension task. These errors were hidden before
this plan because `happyterminals-dsl` had 4 of its own clippy errors that
prevented the model-viewer example from being clippy-checked.

The DSL errors were fixed in this plan (those WERE blocking the current task's
verify step). The model-viewer errors are left for a future pass.

- **examples/model-viewer/main.rs:10:37** — `clippy::doc_markdown` — "InputMap"
  needs backticks in doc comment.
- **examples/model-viewer/main.rs:47:7** — `clippy::too_many_lines` — `async fn main`
  is 146 lines (limit 100); needs decomposition or an `#[allow]`.
- **examples/model-viewer/main.rs:55:9** — `clippy::let_and_return` — redundant
  `let current = …; current` pattern.
- **examples/model-viewer/main.rs:117:22** and **118:22** — `clippy::cast_lossless` —
  `u16 as f32` should use `f32::from(...)`.

**Owner:** Phase 2.3 / Phase 2.5 (author of the model-viewer example).
**Suggested action:** Add a dedicated "lint cleanup before v1 publish" plan in
Phase 03.5 (crates.io publish) so the public release is clippy-clean.
**Impact on Plan 03.4-01:** none — the prelude extension itself is clippy-clean,
compiles, and the prelude compile-test passes. The downstream model-viewer
example has been clippy-dirty since its creation; discovering it now is a side
effect of fixing the DSL errors that previously masked it.

## From Plan 03.4-03 (json-loader example)

### Pre-existing `cargo fmt --all -- --check` failures across examples/lib

Discovered while running the plan-level `cargo fmt --all -- --check` gate for
json-loader. The json-loader file itself formats cleanly under
`rustfmt --edition 2024 --check`. The following pre-existing rustfmt diffs are
NOT caused by this plan — they pre-date Plan 03.4-03 and were reverted from the
working tree to keep this plan's commit scoped to its own file:

- `crates/happyterminals/src/lib.rs` — rustfmt wants sorted import groups
- `crates/happyterminals/examples/color-test/main.rs` — formatting drift
- `crates/happyterminals/examples/model-viewer/main.rs` — import ordering
- `crates/happyterminals/examples/particles/main.rs` — formatting drift
- `crates/happyterminals/examples/static_grid.rs` — formatting drift
- `crates/happyterminals/examples/transitions/main.rs` — formatting drift

**Suggested action:** Include workspace-wide `cargo fmt --all` as a single
bulk-format commit in the Phase 03.5 pre-publish lint cleanup plan.
**Impact on Plan 03.4-03:** none — json-loader/main.rs is rustfmt-clean on its
own (verified with `rustfmt --edition 2024 --check`). The workspace-wide fmt
drift is out of scope for this plan's SCOPE BOUNDARY.
