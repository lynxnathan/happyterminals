# Phase 2.2 — Deferred Items

Discovered during Plan 02.2-01 execution. Out of scope (SCOPE BOUNDARY rule
from execute-plan.md) — pre-existing issues in unrelated files.

## Pre-existing `cargo clippy --tests` errors in backend-ratatui

Verified to pre-exist on commit `84803fc` (before Task 1 of Plan 02.2-01).
26+ clippy errors surface only with `--all-targets` / `--tests` flag; plain
`cargo clippy -p happyterminals-backend-ratatui -- -D warnings` (lib-only)
is clean.

Affected files (all in `crates/happyterminals-backend-ratatui/tests/`):

- `event_signals.rs` — `missing_docs_in_private_items` at crate level.
- `one_cell_change.rs` — `missing_docs_in_private_items` + 12 `unwrap_used`.
- `scene_bytes_test.rs` — 1 `doc_markdown` + multiple `unwrap_used`.
- `e2e_phase_1_1.rs` — 2 `doc_markdown` + ~20 `unwrap_used` + 1 `uninlined_format_args`.

**Why deferred:** These lints apply only when building tests, and the failures
are not caused by Plan 02.2-01 changes. Fixing them is a separate hygiene task
— likely a small hotfix plan or rolled into Phase 2.2 Plan 02 cleanup.

**Not blocking:**
- `cargo test -p happyterminals-backend-ratatui` — green (lints don't fire at
  test build-time under plain `cargo test`; they fire only under
  `cargo clippy --tests`).
- Plan 02.2-01 verification gates (lib clippy + unit tests) — clean.
