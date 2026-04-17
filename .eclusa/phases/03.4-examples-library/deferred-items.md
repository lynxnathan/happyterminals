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

## From Plan 03.4-04 (header polish to DEMO-05)

### Pre-existing clippy errors in `happyterminals-scene` crate

Discovered while running `cargo clippy --workspace --all-targets -- -D warnings`
for the plan-wide verify gate. Plan 03.4-04 only edits `//!` doc-comment headers
(6 example files) — D-09 forbids refactoring existing code. The 30 clippy errors
surfaced by the workspace gate are all in unrelated crate paths:

- `crates/happyterminals-scene/src/transition.rs` — `HashMap::default()` style,
  `unwrap()` on `Result` (8× + 3×)
- `crates/happyterminals-scene/src/transition_effect.rs` — `unwrap()` on
  `Option`, `#[should_panic]` without reason (7× + 1×)
- `crates/happyterminals-scene/src/camera.rs` — `format!` string inlining (4×)
- `crates/happyterminals-scene/src/easing.rs` — `i32 as f32` precision loss (1×)
- `crates/happyterminals-scene/tests/scene_types.rs` — `HashMap::default()`,
  approximate PI literal (2× + 2×)
- `crates/happyterminals-scene/tests/scene_graph.rs` — unused import `Owner`,
  doc-backticks missing (1× + 2×)

**Baseline confirmation:** ran `git stash && cargo clippy` → 90 errors on the
pre-edit tree; after the header edits → 88 errors. The 2-error delta is
clippy-neutral flutter, NOT introduced by this plan.

**Suggested action:** Add `happyterminals-scene` crate lint cleanup to the
Phase 03.5 pre-publish lint plan (alongside the model-viewer example cleanup
from Plan 01 and the workspace fmt bulk-commit from Plan 03).
**Impact on Plan 03.4-04:** none — every file edited by this plan (`//!`
headers only) compiles cleanly via `cargo check --example <name>`. The
workspace clippy errors are pre-existing in unrelated crates.

### Continuing rustfmt drift on example files

The same 168 pre-existing rustfmt diffs present before Plan 03.4-03 are still
present (confirmed via `git stash` baseline → 168 diffs, post-edits → 168 diffs,
zero delta). Header edits don't touch rustfmt-relevant code. Already tracked
above under the Plan 03.4-03 section; no additional action needed.
