# Phase 02.1 — Deferred Items

Items observed during execution but deemed out-of-scope for this phase.
Logged per execute-plan.md scope-boundary rule.

## Pre-existing clippy warnings on `--all-targets` (not on `plan-required` gate)

Discovered during Plan 02.1-02 Task 1 verification. `cargo clippy -p
happyterminals-renderer -p happyterminals-backend-ratatui -- -D warnings`
(the plan's prescribed gate, lib-only) is clean. `--all-targets` surfaces
test-file warnings that predate this phase.

### `crates/happyterminals-renderer/tests/obj_corpus.rs`
- Line 3, 16: "item in documentation is missing backticks".
- Line 39: "match for destructuring a single pattern — consider if let".
- **Landed in:** Plan 02.1-01 (Task 3).
- **Owner:** `/eclusa:verify-work` or a follow-up doc-cleanup plan.

### `crates/happyterminals-renderer/src/cube.rs:157`
- "casting `usize` to `f32` causes a loss of precision".
- Appears in the pre-existing `vertices_centered_at_origin` test
  (`Cube::VERTICES.len() as f32`). Predates this phase.
- **Owner:** micro-PR — add a localized `#[allow(clippy::cast_precision_loss)]`
  on that test.

### `crates/happyterminals-backend-ratatui/tests/*` (e2e_phase_1_1.rs, one_cell_change.rs, event_signals.rs)
- ~30 `unwrap_used`, `missing_docs`, `format!`-style warnings.
- These test files predate workspace lint tightening. Lint-gate for the
  backend crate's *library* code is clean (`--lib` run passes).
- **Owner:** dedicated "tighten test-file lints" cleanup plan.
