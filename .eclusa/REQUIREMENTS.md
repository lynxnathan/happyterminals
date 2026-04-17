# Requirements: happyterminals v2.0

**Defined:** 2026-04-17
**Milestone:** v2.0 Compositor + v1 Release
**Core Value:** Terminal art should feel like magic, not plumbing.

---

## Scene Transitions

- [ ] **SCENE-04**: Full TransitionManager — Scene A → B via named effect (dissolve, slide, etc.) with outgoing owner disposed cleanly
- [ ] **SCENE-06**: At least 3 built-in transition effects (dissolve, slide-left, fade-to-black) usable by name
- [ ] **SCENE-07**: Transition triggered programmatically and via InputMap action

## JSON Recipes

- [ ] **DSL-04**: JSON recipe loader produces SceneIr identical to Rust builder (round-trip property test)
- [ ] **DSL-06**: JSON schema generated via schemars, validated via jsonschema before loading
- [ ] **DSL-07**: Versioned `$version` field in JSON schema
- [ ] **DSL-05**: Effect names resolved through static registry (no eval, no shell-out)
- [ ] **DSL-08**: Mesh paths sandboxed; ANSI-injection stripping on user-provided strings

## Examples Library

- [ ] **REL-04**: 5+ runnable examples (mesh-viewer, particles, transitions, JSON loader, text-reveal)
- [ ] **DEMO-05**: Each example has header comment explaining what it demonstrates

## crates.io Release

- [ ] **REL-01**: All 7 crates have complete Cargo.toml metadata
- [ ] **REL-02**: CHANGELOG.md with Keep-a-Changelog v1.0.0 entry
- [ ] **REL-05**: cargo-semver-checks passes on every crate
- [ ] **REL-06**: docs.rs builds every crate with all features
- [ ] **REL-07**: cargo publish --dry-run succeeds for all 7 crates in order

## Tech Debt Closure

- [x] **REND-11**: Renderer::draw() accepts &dyn Camera (not &OrbitCamera)

---

## Future (M4: Python Bindings)

- PY-01 through PY-10 (PyO3 cdylib, sync run(), GIL handling, abi3 wheels, PyPI publish)

## Out of Scope

- Audio-reactive, AI scene-gen, GLSL→ASCII, live REPL, multi-terminal, WASM — 999.x backlog
- Haskell bindings — removed permanently

---

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| REND-11 | Phase 3.1 | Complete |
| SCENE-04 | Phase 3.1 | Pending |
| SCENE-06 | Phase 3.1 | Pending |
| SCENE-07 | Phase 3.1 | Pending |
| DSL-04 | Phase 3.2 | Pending |
| DSL-06 | Phase 3.2 | Pending |
| DSL-07 | Phase 3.2 | Pending |
| DSL-05 | Phase 3.3 | Pending |
| DSL-08 | Phase 3.3 | Pending |
| REL-04 | Phase 3.4 | Pending |
| DEMO-05 | Phase 3.4 | Pending |
| REL-01 | Phase 3.5 | Pending |
| REL-02 | Phase 3.5 | Pending |
| REL-05 | Phase 3.5 | Pending |
| REL-06 | Phase 3.5 | Pending |
| REL-07 | Phase 3.5 | Pending |

---
*Created: 2026-04-17 for milestone v2.0*
*Traceability updated: 2026-04-17 by eclusa-roadmapper*
