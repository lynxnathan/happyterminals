---
eclusa_state_version: 1.0
milestone: v2.0
milestone_name: exit gates
status: executing
stopped_at: Completed 03.4-04-PLAN.md (DEMO-05 header polish)
last_updated: "2026-04-17T22:21:07.159Z"
last_activity: 2026-04-17
progress:
  total_phases: 5
  completed_phases: 3
  total_plans: 11
  completed_plans: 10
  percent: 82
---

# Project State

## Project Reference

See: `.eclusa/PROJECT.md` (updated 2026-04-17)

**Core value:** Terminal art should feel like magic, not plumbing.
**Current focus:** Phase 03.4 — examples-library

## Current Position

Phase: 03.4 (examples-library) — EXECUTING
Plan: 5 of 5
Status: Ready to execute
Last activity: 2026-04-17

Progress: [████████████████░░░░] 82% (v1.0 complete, v2.0 Phase 03.4 in progress — json-loader shipped)

## Performance Metrics

**Velocity (v1.0):**

- Total plans completed: 37
- Total phases completed: 12
- Average plan duration: ~5.5 min

**Recent Trend (last 5 plans):**

- 02.4-P01: 8min, 02.4-P02: 3min, 02.5-P02: 3min
- Trend: Stable / Improving

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions logged in PROJECT.md Key Decisions table. Recent:

- v1.0 shipped with MSRV 1.88 (upgraded from 1.86 during Phase 2.5)
- TransitionManager scaffold exists from Phase 1.4 (Owner disposal semantics defined)
- Community validated z-axis spatial paradigm -- transitions are highest-value v2.0 feature
- Camera trait with 3 implementations (Orbit/FreeLook/FPS) shipped in Phase 2.3; REND-11 refactors Renderer::draw to accept &dyn Camera
- load_recipe returns (SceneIr, CameraConfig) tuple, not Scene -- avoids Scene validation at recipe load time
- RecipeError kept in DSL crate, not extending SceneError
- JSON props stored as PropValue::Static(Box<serde_json::Value>)
- scene_ir_to_recipe exported in prelude alongside load_recipe for round-trip utility
- [Phase 03.3]: sanitize_path is pure-string (no canonicalize): avoids symlink surprises and non-existent-file errors
- [Phase 03.3]: ANSI stripping is a byte-level scanner, not a regex — zero new dependencies, UTF-8 safe because ESC bytes are ASCII
- [Phase 03.3]: load_recipe left unchanged; load_recipe_sandboxed is the new default for untrusted input
- [Phase 03.3]: Sandboxed mesh paths store the cleaned relative path, not the joined path; downstream keeps asset-root-relative lookup
- [Phase 03.4]: Phase 03.4 Plan 01: happyterminals::prelude mirrors happyterminals-dsl::prelude for JSON/sandbox surface (load_recipe_sandboxed, SandboxConfig, EffectRegistry) — single-import pattern preserved for downstream examples
- [Phase 03.4]: Phase 03.4 Plan 01: 5 pre-existing clippy errors in examples/model-viewer/main.rs deferred to Phase 03.5 pre-publish lint cleanup (SCOPE BOUNDARY — out of current plan scope)
- [Phase 03.4]: Phase 03.4 Plan 02: text-reveal hero example ships — tachyonfx fade_from/sweep_in/coalesce bounded to a title rect via TachyonAdapter::with_area, over a slowly rotating bunny; Space=replay, Tab=cycle. 189 LOC, DEMO-05 header complete.
- [Phase 03.4]: Phase 03.4 Plan 02: tachyonfx added as happyterminals dev-dependency — raw tachyonfx::fx constructors needed for bounded effects; Rust transitive-dep names not exposed, so direct dep is required. Scoped to [dev-dependencies] keeps library surface unchanged.
- [Phase 03.4]: Phase 03.4 Plan 03: json-loader headline example ships — 102 LOC, load_recipe_sandboxed with pinned asset_root, Scene::new + run_scene, commented-out RecipeError::PathTraversal demo, single happyterminals::prelude::* import validates Plan 01 end-to-end.
- [Phase 03.4]: Phase 03.4 Plan 03: Workspace-wide rustfmt drift across 6 unrelated files (model-viewer, color-test, particles, transitions, static_grid, lib.rs) deferred to Phase 03.5 pre-publish lint cleanup (SCOPE BOUNDARY).
- [Phase 03.4]: Phase 03.4 Plan 04: all 6 pre-existing example headers (spinning-cube, model-viewer, particles, transitions, color-test, static_grid) brought to DEMO-05 canonical 5-section shape; particles gains zero-per-frame-allocation brag per D-10; color-test and static_grid carry verbatim 'Developer utility — not a demo' marker per D-02; zero non-header code changed (D-09 honored).
- [Phase 03.4]: Phase 03.4 Plan 04: workspace clippy/fmt drift (88 pre-existing errors in happyterminals-scene, 168 fmt diffs) confirmed neutral via git-stash baseline and routed to Phase 03.5 pre-publish lint plan (SCOPE BOUNDARY — D-09 forbids refactoring non-header code).

### Pending Todos

None yet for v2.0.

### Blockers/Concerns

- Phase 2.3 Plan 04 (human verification of model-viewer) and Phase 2.5 Plan 01 (STL loader) still marked incomplete in v1.0 roadmap -- confirm closure before v2.0 work begins
- schemars 1.2 and jsonschema 0.46 pinned since Phase 0 but never exercised -- version drift check needed in Phase 3.2 research

## Session Continuity

Last session: 2026-04-17T22:21:07.155Z
Stopped at: Completed 03.4-04-PLAN.md (DEMO-05 header polish)
Resume file: None
Next command: Execute 03.4-04-PLAN.md
