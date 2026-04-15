# State: happyterminals

## Project Reference

See: `.eclusa/PROJECT.md` (updated 2026-04-14)

**Core value:** Terminal art should feel like magic, not plumbing.
**Current focus:** Milestone 0 — COMPLETE (Phase 0 shipped 2026-04-14). Next: Milestone 1 — Spinning Cube Demo, starting with Phase 1.0 (Reactive Core).
**Next command:** `/eclusa:plan-phase 1.0`

---

## Current Milestone

**M0 — Workspace Cleanup** (prerequisite to all feature work).

### Phase under planning next

- **Phase 0** — Workspace Hygiene & Foundation. See `.eclusa/ROADMAP.md` §"Milestone 0".

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
| **M1 Spinning Cube Demo** | **Next — Phase 1.0** | 1.0 → 1.5 | `examples/spinning-cube/` <100 LOC, cross-terminal verified, 1-cell → ~10 bytes |
| M2 Renderer Depth | Not started | 2.1–2.4 | OBJ + color pipeline + particles + resize hardening |
| M3 Compositor + JSON + Release | Not started | 3.1–3.5 | Transitions + JSON recipes + 7 crates published |
| M4 Python Bindings (**FINAL**) | Not started | 4.1–4.4 | `pip install happyterminals` cross-platform |

---

## Open Questions (carry through planning)

Tracked in `.eclusa/ROADMAP.md` §"Open Questions". The eclusa-planner will pick these up per phase:

1. **Q1 `Memo<T>: PartialEq` bound** — resolve in Phase 1.0.
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
*Last updated: 2026-04-14 after initialization*
