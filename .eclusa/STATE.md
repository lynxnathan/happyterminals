---
eclusa_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: unknown
stopped_at: Completed 02.5-02-PLAN.md
last_updated: "2026-04-17T05:53:11.753Z"
progress:
  total_phases: 13
  completed_phases: 8
  total_plans: 37
  completed_plans: 28
---

# State: happyterminals

## Project Reference

See: `.eclusa/PROJECT.md` (updated 2026-04-14)

**Core value:** Terminal art should feel like magic, not plumbing.
**Current focus:** Phase 02.4 — Particles + Per-Frame Allocation Bench
**Next command:** `/eclusa:autonomous`

---

## Current Milestone

**M2 — Renderer Depth + Cross-Terminal Polish** (M0 and M1 complete as of 2026-04-15).

### Phase under planning next

- **Phase 2.1** — OBJ Mesh Loading (`tobj 4.0`, Result-typed, 10+ file corpus, bunny/cow/teapot viewer). See `.eclusa/ROADMAP.md` §"Phase 2.1: OBJ Mesh Loading".

### Exit criteria (recap)

- Clean `cargo build --workspace` on Rust 1.86, zero warnings.
- `cargo tree -d` zero duplicates.
- Dual LICENSE files + SPDX on every crate.
- `happyterminals` reserved on crates.io + PyPI (plus variants).
- CI baseline green (fmt, clippy `-D warnings`, tests, docs, duplicate-dep scan, unused-dep scan, doc-lint).
- No forbidden strings (`tui-vfx`, `Haskell bindings`, `pyo3-asyncio`, `cgmath`, `tui-rs`) outside approved rationale sections.

---

## Milestone Plan

| Milestone | Status | Phases | Exit |
|-----------|--------|--------|------|
| **M0** Workspace Cleanup | **COMPLETE (2026-04-14)** | Phase 0 | HYG-01..05, 07..09 satisfied (HYG-06 deferred) |
| **M1** Spinning Cube Demo | **COMPLETE (2026-04-15)** | 1.0 → 1.5 | `examples/spinning-cube/` 43 LOC, cross-terminal verified, 165 tests passing |
| **M2** Renderer Depth | **Next — Phase 2.1** | 2.1–2.4 | OBJ + color pipeline + particles + resize hardening |
| M3 Compositor + JSON + Release | Not started | 3.1–3.5 | Transitions + JSON recipes + 7 crates published |
| M4 Python Bindings (**FINAL**) | Not started | 4.1–4.4 | `pip install happyterminals` cross-platform |

---

## Open Questions (carry through planning)

Tracked in `.eclusa/ROADMAP.md` §"Open Questions". The eclusa-planner will pick these up per phase:

1. **Q1 `Memo<T>: PartialEq` bound** — **RESOLVED** in Phase 1.0-02 (2026-04-15): `PartialEq` is always-on. Secondary finding: `Memo<T>` also requires `T: Send + Sync` in v0.0.0 (reactive_graph 0.2.13's `SyncStorage`-default `Memo<T>` imposes it; pre-authorized in plan 01.0-02 §decisions.2). Potential future relaxation via a `LocalStorage`-backed variant is tracked.
2. **Q2 Async runtime** (tokio vs smol) — resolve in Phase 1.1.
3. **Q3 `Effect` name clash** — recommended resolution: tachyonfx's becomes `Fx`. Resolve in Phase 1.2 before any pipeline consumer.
4. **Q4 Wide-char rendering polish** — Phase 1.1 ships grapheme + width fields; wide-cell edge-case polish deferrable.
5. **Q5 JSON schema versioning** — Phase 3.2.
6. **Q6 Python sync vs asyncio primary surface** — Phase 4.2 planning (default: sync `run()` first per ARCH §9.4).
7. **Q7 Grid-as-ratatui-Buffer layout compat** — 1–2 day spike inside Phase 1.1.
8. **Q8 Roadmapper ordering philosophy** — RESOLVED: bottom-up with vertical-slice pulls (ARCH §11.2).

---

## Key Decisions (frozen)

From PROJECT.md §"Key Decisions". Do not revisit without an explicit re-planning conversation:

- tachyonfx (not tui-vfx) is the effects foundation.
- Re-implement 3D renderer fresh, voxcii-inspired (no fork).
- SolidJS-style fine-grained signals (no VDOM).
- Rust core + Python bindings only — **no Haskell bindings**.
- Python bindings are the **LAST milestone**.
- Milestone 1 exits with the spinning cube demo.
- Public release from day one, dual MIT OR Apache-2.0.
- Phase 5 "fun" items (audio-reactive, AI scene-gen, GLSL transpile, live REPL, multi-terminal) → 999.x backlog.
- Scene DSL takes cues from **react-three-fiber** (tree-of-nodes with Signal props, fine-grained reactivity).

---

## Workflow Configuration

From `.eclusa/config.json`:

- Mode: **YOLO** (auto-approve)
- Granularity: **Standard** (5–8 phases per milestone, 3–5 plans each)
- Parallelization: **Enabled**
- Research before each phase: **Yes**
- Plan Check: **Yes**
- Verifier: **Yes**
- Model profile: **Balanced** (Sonnet for most agents)
- Commit planning docs to git: **Yes**

---

## Artifacts

| File | Purpose | Last updated |
|------|---------|--------------|
| `.eclusa/PROJECT.md` | Living project context | 2026-04-14 (init) |
| `.eclusa/REQUIREMENTS.md` | 69 v1 requirements across 10 groups | 2026-04-14 (init) |
| `.eclusa/ROADMAP.md` | 4 milestones, M1 detailed, M2–M4 sketched | 2026-04-14 (init) |
| `.eclusa/research/STACK.md` | Stack research (crates + versions, anti-recs) | 2026-04-14 |
| `.eclusa/research/FEATURES.md` | Feature landscape (table stakes / diff / anti) | 2026-04-14 |
| `.eclusa/research/ARCHITECTURE.md` | Six-crate split, data flow, IR design | 2026-04-14 |
| `.eclusa/research/PITFALLS.md` | 33 pitfalls, phase-mapped | 2026-04-14 |
| `.eclusa/research/SUMMARY.md` | Synthesized research summary | 2026-04-14 |
| `.eclusa/config.json` | Workflow preferences | 2026-04-14 |
| `.eclusa/STATE.md` | This file — project memory / current focus | 2026-04-14 |
| `project.eclusa` | Machine-readable project identity + stances | 2026-04-14 |

---
## Session Continuity

Last session: 2026-04-17T05:53:11.748Z
Stopped at: Completed 02.5-02-PLAN.md
Resume file: None
Pending decision carried: model-toggler (bunny/cow/teapot viewer) resolved by pulling Phase 2.1 forward (option B from 2026-04-15 session)

---

## Performance Metrics

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 01.1 | 02 | 3min | 2 | 6 |

---
*Last updated: 2026-04-15 — Plan 01.1-02 complete. TerminalGuard RAII, InputEvent mapping, FrameSpec shipped.*
| Phase 01.1 P03 | 5min | 3 tasks | 11 files |
| Phase 01.2 P01 | 5min | 2 tasks | 8 files |
| Phase 01.2 P02 | 7min | 2 tasks | 5 files |
| Phase 01.3 P01 | 4min | 2 tasks | 7 files |
| Phase 01.3 P02 | 9min | 2 tasks | 10 files |
| Phase 01.4 P01 | 4min | 2 tasks | 12 files |
| Phase 01.4 P02 | 5min | 2 tasks | 10 files |
| Phase 02.1 P01 | 8 min | 3 tasks | 16 files |
| Phase 02.1 P02 | 6 min | 2 tasks | 6 files |
| Phase 02.1 P03 | 3 min | 2 tasks | 5 files |
| Phase 02.2 P01 | 16 min | 3 tasks | 7 files |
| Phase 02.2 P02 | 4 min | 2 tasks | 6 files |
| Phase 02.2 P03 | 7 min | 3 tasks | 10 files |
| Phase 02.4 P01 | 8min | 2 tasks | 4 files |
| Phase 02.4 P02 | 3min | 1 tasks | 3 files |
| Phase 02.5 P02 | 3min | 1 tasks | 1 files |
