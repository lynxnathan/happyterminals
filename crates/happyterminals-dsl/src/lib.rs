//! # happyterminals-dsl
//!
//! Declarative scene builder for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! A `react-three-fiber`-shaped tree of typed nodes with props that can be plain
//! values, `Signal<T>`, or `Memo<T>`. The builder produces a validated [`Scene`]
//! via [`scene()`] -> chain -> [`SceneBuilder::build()`].
//!
//! # Quick Start
//!
//! ```ignore
//! use happyterminals_dsl::prelude::*;
//!
//! let scene = scene()
//!     .camera(OrbitCamera::default())
//!     .layer(0, |l| l.cube().position(vec3(0., 0., 0.)))
//!     .build()?;
//! ```

#![forbid(unsafe_code)]

pub mod builder;
pub mod json;
pub mod node_builder;
pub mod prelude;
pub mod sandbox;

pub use builder::SceneBuilder;

/// Entry point for the DSL builder chain.
///
/// Returns a fresh [`SceneBuilder`] ready for method chaining.
///
/// # Example
///
/// ```ignore
/// let scene = scene()
///     .camera(OrbitCamera::default())
///     .layer(0, |l| l.cube())
///     .build()?;
/// ```
#[must_use]
pub fn scene() -> SceneBuilder {
    SceneBuilder::new()
}
