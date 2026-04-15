//! # happyterminals-dsl
//!
//! Declarative scene builder for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! A `react-three-fiber`-shaped tree of typed nodes with props that can be plain
//! values, `Signal<T>`, or `Memo<T>`. Also ships the JSON recipe loader that
//! validates input against a `schemars`-generated schema via `jsonschema`, then
//! produces the same `SceneIr` as the Rust builder path.
//!
//! Phase 0 scaffolding — no public types yet. Rust builder lands in Phase 1.4;
//! JSON recipes in Phases 3.2–3.4.
