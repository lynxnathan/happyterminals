# happyterminals-core

Fine-grained reactive runtime for the [happyterminals](https://github.com/lynxnathan/happyterminals)
project. Wraps [`reactive_graph 0.2.13`](https://docs.rs/reactive_graph/0.2.13/)
behind a happyterminals-owned public API.

Status: **v0.0.0 (pre-release).** Not yet published. Semver discipline
begins at v0.1.0 (Phase 3.5).

## Overview

- `Signal<T>`, `Memo<T>`, `Effect` — the reactive primitives.
- `Owner`, `create_root`, `on_cleanup` — scope + RAII disposal tree.
- `batch(|| ...)` — coalesces signal writes into one propagation pass.
- `SignalSetter<T>` — `Send` handle for cross-thread writes (mpsc-queued,
  drained on the owning thread via `Signal::drain_setter_queue`).
- `Clock` + `Rng` traits with `SystemClock` / `ThreadRng` (prod) and
  `ManualClock` / `SeededRng` (behind `--features test-util`) for tests.

All reactive types are `!Send + !Sync` (single-threaded runtime).

See the crate-level doc comment in `src/lib.rs` for the full threading
contract, `PartialEq` policy, and cycle-detection behavior.

## Testing strategy (Phase 1.0 status)

Phase 1.0 landed:

- 5 integration tests in `tests/` (disposal, cycle-detect, signal_setter,
  untracked, batch) — all green.
- 2 Memo-exercising integration tests (`diamond`, `transitions_10k`) gated
  behind `#[ignore]` pending a `reactive_graph` `ImmediateEffect + Memo`
  deadlock fix. See `01.0-05-SUMMARY.md` §"Deferred Issues" for the full
  reproduction + escalation path.
- 2 property tests (`proptest_set_get`, `proptest_batch`).
- 1 compile-time test (`send_sync`) asserting the `!Send + !Sync` surface
  via [`static_assertions`](https://docs.rs/static_assertions/).
- 1 criterion bench (`benches/memo_eq_skip.rs`) with an automated
  `< 1µs` gate on `mean.point_estimate` parsed from
  `target/criterion/<bench>/base/estimates.json`.
- `cargo doc --no-deps -- -D warnings` passes — every public item has a
  `///` doc comment.

### Doc-test deferral (decision recorded 2026-04-15)

CONTEXT.md §"Testing strategy" for Phase 1.0 specified "Doc-tests on every
public item where feasible." In v0.0.0 we deliberately **do NOT** run
doc-tests on most reactive types (`Signal`, `Memo`, `Effect`, `Owner`).
Reason: the reactive API requires a `create_root` scope for almost every
meaningful example, and `cargo test --doc`'s one-example-one-compile
model makes the boilerplate overwhelming (the setup is longer than the
illustration). Running integration tests in `tests/` delivers the same
confidence at a fraction of the per-example cost.

- Items where doc-tests run today: none (all examples are ``` `ignore` ```d).
- **Later phases** (1.4+ / HYG-08) may revisit this: if a specific API's
  docs would benefit from runnable examples (e.g., a frequently-misused
  method), add a targeted doc-test then rather than trying to cover the
  whole surface at once.

The comprehensive test suite in `tests/` covers REACT-01..09 (REACT-10
coverage deferred pending the Memo deadlock fix — see SUMMARY).

## License

Dual-licensed under MIT OR Apache-2.0. See workspace `Cargo.toml`.
