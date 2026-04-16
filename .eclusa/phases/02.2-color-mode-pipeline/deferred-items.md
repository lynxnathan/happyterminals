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

## Plan 02.2-02 — additional pre-existing `--all-targets` findings

Verified to pre-exist by `git stash && cargo clippy --workspace --all-targets`
check prior to Task 2 commit on Plan 02.2-02.

Workspace-wide `cargo clippy --workspace --all-targets -- -D warnings` surfaces
~32 errors in:

- `crates/happyterminals-core/src/grid.rs` + `src/lib.rs` — `unwrap_used` (15x),
  `cast_possible_truncation` (1x), `no_effect_underscore_binding` (1x).
- `crates/happyterminals-scene/tests/scene_graph.rs` + `scene_types.rs` —
  `default_trait_access`, `approx_constant`, unused `Owner` import.
- `crates/happyterminals-renderer/tests/obj_corpus.rs` — `doc_markdown`,
  `single_match`.
- `crates/happyterminals-pipeline/tests/smoke.rs` — unused `Effect` import.
- `crates/happyterminals/src/lib.rs` — `items_after_statements` on the
  pre-existing `fn _check_types()` nested inside the `prelude_reexports_compile`
  test (pre-existed at line 66 before Plan 02.2-02; now at line 69 because
  new `ColorMode` re-export line pushed it down).

**Why deferred:** Identical rationale to the Plan 02.2-01 entry above — the
scope boundary rule limits each plan to fixing issues DIRECTLY caused by its
own changes. These are all pre-existing hygiene items in unrelated files and
tests that pre-date Plan 02.2-02.

**Not blocking for Plan 02.2-02:**
- `cargo build --workspace` — clean.
- `cargo test --workspace` — 249 passed, 6 ignored (37 suites).
- `cargo clippy -p happyterminals-backend-ratatui -- -D warnings` — clean.
- `cargo clippy -p happyterminals --lib -- -D warnings` — clean.
- All grep invariants from Plan 02.2-02 `<verification>` block — pass.

Suggested follow-up: single hygiene hotfix plan that does a workspace-wide
`cargo clippy --fix --workspace --all-targets` sweep, targeting only the
trivially auto-fixable lints, followed by manual review for `unwrap_used`
sites in `happyterminals-core/src/grid.rs` (which need semantic judgment).
