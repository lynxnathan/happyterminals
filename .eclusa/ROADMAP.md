# Roadmap: happyterminals

**Created:** 2026-04-14
**Milestone Roadmap Updated:** 2026-04-17 (v2.0 phases added)
**Granularity:** Standard (5-8 phases per milestone)
**Parallelization:** Enabled
**Core Value:** Terminal art should feel like magic, not plumbing. A tiny declarative scene description should produce cinematic, reactive, cross-terminal output.

---

## Milestones

- **v1.0 Renderer Depth** - Phases 0 through 2.5 (shipped 2026-04-17)
- **v2.0 Compositor + v1 Release** - Phases 3.1 through 3.5 (in progress)
- **v3.0 Python Bindings (FINAL)** - Phases 4.1 through 4.4 (planned)

---

## Phases

<details>
<summary>v1.0 Renderer Depth (Phases 0 - 2.5) -- SHIPPED 2026-04-17</summary>

- [x] **Phase 0: Workspace Hygiene** - Stub-crate dep rot, vendor debris, license + registry plumbing, CI baseline. (completed 2026-04-14)
- [x] **Phase 1.0: Reactive Core** - Signal/Memo/Effect/Owner/batch/untracked/SignalSetter with clean disposal and cycle detection. (completed 2026-04-15)
- [x] **Phase 1.1: Grid + Ratatui Backend** - Grapheme-correct Grid, panic-safe TerminalGuard, tokio::select loop, 1-cell-bytes test harness. (completed 2026-04-15)
- [x] **Phase 1.2: Pipeline + tachyonfx** - Effect trait, Pipeline executor, TachyonAdapter, Fx rename, 10+ effects. (completed 2026-04-15)
- [x] **Phase 1.3: Minimal Renderer** - Z-buffer, cell-aspect projection, reversed-Z, shading ramp, Cube, orbit camera. (completed 2026-04-15)
- [x] **Phase 1.4: Scene IR + Rust DSL** - SceneIr, SceneGraph, react-three-fiber builder, signal-bindable props. (completed 2026-04-15)
- [x] **Phase 1.5: Spinning Cube Demo** - <100 LOC example, README, cross-terminal matrix -- M1 EXIT. (completed 2026-04-15)
- [x] **Phase 2.1: OBJ Mesh Loading** - tobj 4.0, Result-typed, model-viewer with bunny/cow/teapot. (completed 2026-04-15)
- [x] **Phase 2.2: Color-Mode Pipeline** - ColorMode cascade, flush-time downsample, NO_COLOR, insta snapshots. (completed 2026-04-16)
- [x] **Phase 2.3: Input Action System + Camera Modes** - Godot/Unreal InputMap, mouse drag, FreeLook/FPS cameras. (completed 2026-04-16)
- [x] **Phase 2.4: Particles + Per-Frame Alloc Bench** - Pool-based particles, zero per-frame allocation, snow-over-bunny. (completed 2026-04-16)
- [x] **Phase 2.5: Resize Hardening + MSRV + STL** - Skip-frame resize, STL loader, MSRV 1.88 CI. (completed 2026-04-17)

**v1.0 stats:** 12 phases, 37 plans, 365 tests, 13.6K LOC Rust, MSRV 1.88.

</details>

### v2.0 Compositor + v1 Release (in progress)

**Milestone Goal:** Complete the declarative surface (scene transitions, JSON recipes, schema validation, security sandbox) and publish v1 to crates.io with 5+ examples.

- [ ] **Phase 3.1: Camera Refactor + Scene Transitions** - Polymorphic Camera trait on Renderer::draw, full TransitionManager with 3+ built-in effects
- [ ] **Phase 3.2: JSON Recipe Loader + Schema** - JSON-to-SceneIr round-trip, schemars schema generation, jsonschema validation, versioned schema
- [ ] **Phase 3.3: JSON Sandbox** - Static effect registry, path sandboxing, ANSI-injection stripping
- [ ] **Phase 3.4: Examples Library** - 5+ runnable examples with documentation headers
- [ ] **Phase 3.5: crates.io Publish** - 7 crates with metadata, CHANGELOG, semver-checks, docs.rs, dry-run

### v3.0 Python Bindings (planned)

- [ ] **Phase 4.1: PyO3 cdylib** - happyterminals-py crate, Python API mirroring Rust builder
- [ ] **Phase 4.2: Sync run() + GIL** - Primary Python entry point, GIL release, SignalSetter channel
- [ ] **Phase 4.3: Wheels + PyPI** - abi3 wheels, Trusted Publishing, type stubs
- [ ] **Phase 4.4: Python hello-world + v1 launch** - 10-line Python spinning cube, pip install

---

## Phase Details

### Phase 3.1: Camera Refactor + Scene Transitions
**Goal**: Users can transition between scenes with cinematic spatial effects, and the renderer accepts any camera type polymorphically
**Depends on**: Phase 2.5 (v1.0 shipped, TransitionManager scaffold exists from Phase 1.4)
**Requirements**: REND-11, SCENE-04, SCENE-06, SCENE-07
**Success Criteria** (what must be TRUE):
  1. `Renderer::draw()` accepts `&dyn Camera` -- any code using `&OrbitCamera` directly has been migrated, and user-defined Camera implementations work without modifying the renderer
  2. Calling `transition_manager.transition_to(scene_b, "dissolve")` plays a visible dissolve effect from Scene A to Scene B, with Scene A's reactive Owner disposed cleanly (no leaked signals/effects)
  3. At least 3 named transition effects (dissolve, slide-left, fade-to-black) are available and visually distinct when triggered
  4. A transition can be triggered both programmatically (`transition_to()`) and via an InputMap action (e.g., pressing a key bound to "next_scene")
  5. After a transition completes, the new scene's signals and effects are fully operational -- reactive updates continue without interruption
**Plans**: TBD
**UI hint**: yes

### Phase 3.2: JSON Recipe Loader + Schema
**Goal**: Users (and LLMs) can describe scenes in JSON files that produce identical SceneIr to the Rust builder, with schema validation catching errors before loading
**Depends on**: Phase 3.1 (transitions available for JSON scene descriptions)
**Requirements**: DSL-04, DSL-06, DSL-07
**Success Criteria** (what must be TRUE):
  1. A JSON recipe file loaded via the recipe loader produces a `SceneIr` byte-identical to the same scene constructed via the Rust builder DSL (verified by round-trip property test)
  2. A JSON schema generated via `schemars` is published alongside the loader, and `jsonschema` validation rejects malformed recipes with human-readable error messages before any scene construction begins
  3. Every JSON recipe includes a `$version` field; loading a recipe with an unsupported version produces a clear error (not a silent misparse)
  4. An LLM (or human) given only the JSON schema can author a valid scene recipe without reading Rust source code
**Plans**: TBD

### Phase 3.3: JSON Sandbox
**Goal**: JSON recipes execute in a security sandbox where effect resolution is static, file paths are constrained, and user-provided strings cannot inject ANSI escapes
**Depends on**: Phase 3.2 (JSON loader exists to be sandboxed)
**Requirements**: DSL-05, DSL-08
**Success Criteria** (what must be TRUE):
  1. Effect names in JSON recipes resolve through a static registry -- attempting to reference an unregistered effect name returns an error (no eval, no shell-out, no dynamic loading)
  2. Mesh file paths in JSON recipes are sandboxed to a configurable asset directory -- a path like `../../etc/passwd` is rejected before any file I/O occurs
  3. User-provided strings that land in Grid cells have ANSI escape sequences stripped -- a JSON recipe containing `\x1b[2J` in a text field renders the literal characters, not a screen clear
**Plans**: TBD

### Phase 3.4: Examples Library
**Goal**: New users can learn the library through 5+ self-contained, documented examples covering the major feature areas
**Depends on**: Phase 3.3 (JSON sandbox complete, so JSON loader example is safe to ship)
**Requirements**: REL-04, DEMO-05
**Success Criteria** (what must be TRUE):
  1. At least 5 runnable examples exist: mesh-viewer, particles, transitions, JSON-loader, and text-reveal (or equivalent coverage of major features)
  2. Every example has a header comment block explaining what it demonstrates, which crate features it exercises, and how to run it
  3. `cargo run --example <name>` works for each example in a fresh checkout with no additional setup beyond `cargo build`
  4. Examples collectively demonstrate: 3D rendering, particle effects, scene transitions, JSON recipe loading, and text/effect composition
**Plans**: TBD
**UI hint**: yes

### Phase 3.5: crates.io Publish
**Goal**: All 7 crates are published to crates.io with complete metadata, documentation, changelog, and semver verification
**Depends on**: Phase 3.4 (examples ship as part of the published crates)
**Requirements**: REL-01, REL-02, REL-05, REL-06, REL-07
**Success Criteria** (what must be TRUE):
  1. All 7 crates (happyterminals-core, -renderer, -pipeline, -scene, -dsl, -backend-ratatui, happyterminals) have complete `Cargo.toml` metadata: description, license, repository, keywords, categories, readme
  2. A `CHANGELOG.md` following Keep-a-Changelog format exists with a complete v1.0.0 entry documenting all shipped features
  3. `cargo semver-checks` passes on every crate with no breaking-change violations
  4. `docs.rs` builds every crate successfully with all features enabled -- no broken doc links, no missing re-exports
  5. `cargo publish --dry-run` succeeds for all 7 crates in dependency order (core first, meta last)
**Plans**: TBD

---

## Dependency DAG (v2.0)

```
Phase 3.1 (Camera refactor + Transitions)
    |
    v
Phase 3.2 (JSON recipe loader + schema)
    |
    v
Phase 3.3 (JSON sandbox)
    |
    v
Phase 3.4 (Examples library)
    |
    v
Phase 3.5 (crates.io publish) --> v2.0 EXIT
```

**Parallelization notes (v2.0):**
- Phases are sequential -- each builds on the previous.
- Within Phase 3.1, the Camera refactor (REND-11) and TransitionManager implementation can run as parallel plans since they touch different modules.
- Within Phase 3.4, individual examples are independent and can be written in parallel.
- Within Phase 3.5, metadata/changelog work and semver-checks/docs.rs verification are parallel.

---

## Coverage Matrix (v2.0)

| Requirement | Phase | Description |
|-------------|-------|-------------|
| REND-11 | Phase 3.1 | Renderer::draw() accepts &dyn Camera |
| SCENE-04 | Phase 3.1 | Full TransitionManager with owner disposal |
| SCENE-06 | Phase 3.1 | 3+ built-in transition effects |
| SCENE-07 | Phase 3.1 | Transition triggered programmatically and via InputMap |
| DSL-04 | Phase 3.2 | JSON recipe loader round-trip with Rust builder |
| DSL-06 | Phase 3.2 | JSON schema via schemars + jsonschema validation |
| DSL-07 | Phase 3.2 | Versioned $version field in JSON schema |
| DSL-05 | Phase 3.3 | Static effect registry (no eval) |
| DSL-08 | Phase 3.3 | Path sandboxing + ANSI-injection stripping |
| REL-04 | Phase 3.4 | 5+ runnable examples |
| DEMO-05 | Phase 3.4 | Header comments on each example |
| REL-01 | Phase 3.5 | Complete Cargo.toml metadata on all 7 crates |
| REL-02 | Phase 3.5 | CHANGELOG.md with v1.0.0 entry |
| REL-05 | Phase 3.5 | cargo-semver-checks passes |
| REL-06 | Phase 3.5 | docs.rs builds all crates |
| REL-07 | Phase 3.5 | cargo publish --dry-run succeeds |

**Coverage stats (v2.0):**
- v2.0 requirements total: 16
- Mapped to phases: 16
- Orphaned: 0

---

## Progress Table

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 0. Workspace Hygiene | v1.0 | 5/5 | Complete | 2026-04-14 |
| 1.0 Reactive Core | v1.0 | 6/6 | Complete | 2026-04-15 |
| 1.1 Grid + Ratatui Backend | v1.0 | 3/3 | Complete | 2026-04-15 |
| 1.2 Pipeline + tachyonfx | v1.0 | 2/2 | Complete | 2026-04-15 |
| 1.3 Minimal Renderer | v1.0 | 2/2 | Complete | 2026-04-15 |
| 1.4 Scene IR + Rust DSL | v1.0 | 2/2 | Complete | 2026-04-15 |
| 1.5 Spinning Cube Demo | v1.0 | 3/3 | Complete -- M1 EXIT | 2026-04-15 |
| 2.1 OBJ Mesh Loading | v1.0 | 3/3 | Complete | 2026-04-15 |
| 2.2 Color-Mode Pipeline | v1.0 | 3/3 | Complete | 2026-04-16 |
| 2.3 Input Action System | v1.0 | 4/4 | Complete | 2026-04-16 |
| 2.4 Particles | v1.0 | 2/2 | Complete | 2026-04-16 |
| 2.5 Resize + MSRV + STL | v1.0 | 2/2 | Complete -- v1.0 EXIT | 2026-04-17 |
| 3.1 Camera Refactor + Transitions | v2.0 | 0/? | Not started | - |
| 3.2 JSON Recipe Loader + Schema | v2.0 | 0/? | Not started | - |
| 3.3 JSON Sandbox | v2.0 | 0/? | Not started | - |
| 3.4 Examples Library | v2.0 | 0/? | Not started | - |
| 3.5 crates.io Publish | v2.0 | 0/? | Not started | - |

*Plan counts populated by `plan-phase` when each phase is decomposed.*

---

## Notes for eclusa-planner

### Per-phase expectations (v2.0)

- **3-5 plans per phase**, each a coherent unit verifiable in one plan cycle.
- **Parallelization enabled** -- call out which plans can run in parallel vs must sequence.
- **Research before each phase: YES.**
- **Plan check + Verifier: YES** per plan.
- **Commit planning docs to git: YES.**

### v2.0-specific context

- **v1.0 shipped:** 7 crates, 13.6K LOC, 365 tests, MSRV 1.88. TransitionManager scaffold exists from Phase 1.4. SceneIr and the Rust builder DSL are stable.
- **Community feedback:** z-axis spatial paradigm validated ("game changer"). Transitions are highest-priority feature.
- **REND-11 tech debt:** `Renderer::draw()` currently takes `&OrbitCamera` directly. Refactoring to `&dyn Camera` early in Phase 3.1 benefits all subsequent phases.
- **Existing crates:** happyterminals-core, -renderer, -pipeline, -scene, -dsl, -backend-ratatui, happyterminals (meta).
- **Workspace deps:** schemars 1.2, jsonschema 0.46 already pinned in workspace Cargo.toml from Phase 0.

### v2.0 exit gates (hard)

1. Scene A -> Scene B transition plays with visible dissolve/slide/fade, outgoing owner disposed.
2. JSON recipe round-trips with Rust builder (property test).
3. JSON sandbox rejects path traversal + ANSI injection.
4. 5+ examples each runnable via `cargo run --example <name>`.
5. `cargo publish --dry-run` succeeds for all 7 crates in order.
6. `cargo semver-checks` green. `docs.rs` builds clean.
7. CHANGELOG.md v1.0.0 entry complete.

---

*Roadmap created: 2026-04-14*
*v2.0 phases added: 2026-04-17*
*Author: eclusa-roadmapper*
*Granularity: Standard*
*Parallelization: Enabled*
