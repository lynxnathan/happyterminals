//! [`TransitionManager`] -- scene-to-scene transitions with named effects.
//!
//! The transition manager maintains a state machine (`TransitionState`) that
//! tracks whether we are idle (displaying a single scene) or transitioning
//! between two scenes. During a transition, both scenes and their owners are
//! held alive; the outgoing owner is disposed only when the transition
//! completes (progress >= 1.0).
//!
//! # Disposal timing invariant
//!
//! Disposal happens synchronously in [`TransitionManager::tick`] when
//! progress reaches 1.0, which is called between frames (not mid-frame).
//! The runtime must ensure `tick` is never called while an Effect from the
//! old scene is executing.

use std::collections::HashMap;
use std::time::Duration;

use happyterminals_core::Owner;
use ratatui_core::buffer::Buffer;

use crate::easing;
use crate::scene::Scene;
use crate::transition_effect::{Dissolve, FadeToBlack, SlideLeft, TransitionEffect};

/// The current state of a [`TransitionManager`].
pub enum TransitionState {
    /// Displaying a single scene.
    Idle {
        /// The current scene.
        scene: Scene,
        /// The reactive owner for the current scene.
        owner: Owner,
    },
    /// Transitioning between two scenes.
    Transitioning {
        /// The outgoing scene.
        from_scene: Scene,
        /// The reactive owner for the outgoing scene.
        from_owner: Owner,
        /// The incoming scene.
        to_scene: Scene,
        /// The reactive owner for the incoming scene.
        to_owner: Owner,
        /// The name of the effect being used.
        effect_name: String,
        /// Time elapsed since the transition started.
        elapsed: Duration,
        /// Total duration of the transition.
        duration: Duration,
        /// Easing function applied to progress.
        easing: fn(f32) -> f32,
    },
}

/// Manages scene-to-scene transitions with named effects and owner disposal.
pub struct TransitionManager {
    state: Option<TransitionState>,
    registry: HashMap<String, Box<dyn TransitionEffect>>,
}

impl TransitionManager {
    /// Creates a new transition manager with the 3 built-in effects registered.
    #[must_use]
    pub fn new() -> Self {
        todo!()
    }

    /// Sets a new scene immediately (no transition). Disposes old owner if present.
    pub fn set_scene(&mut self, _scene: Scene, _owner: Owner) {
        todo!()
    }

    /// Registers a named transition effect.
    pub fn register(&mut self, _name: impl Into<String>, _effect: impl TransitionEffect + 'static) {
        todo!()
    }

    /// Starts a transition to a new scene using the named effect and linear easing.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the effect name is not registered or if not in Idle state.
    pub fn transition_to(
        &mut self,
        _scene: Scene,
        _owner: Owner,
        _effect_name: &str,
        _duration: Duration,
    ) -> Result<(), crate::error::SceneError> {
        todo!()
    }

    /// Starts a transition with a custom easing function.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the effect name is not registered or if not in Idle state.
    pub fn transition_to_with_easing(
        &mut self,
        _scene: Scene,
        _owner: Owner,
        _effect_name: &str,
        _duration: Duration,
        _easing: fn(f32) -> f32,
    ) -> Result<(), crate::error::SceneError> {
        todo!()
    }

    /// Advances the transition by `dt`. If progress reaches 1.0, the old owner
    /// is disposed and the manager returns to Idle with the new scene.
    pub fn tick(&mut self, _dt: Duration) {
        todo!()
    }

    /// Blends the two scene buffers during a transition.
    ///
    /// Returns `true` if a blend was performed (transitioning), `false` if idle.
    pub fn blend(&self, _buf_a: &Buffer, _buf_b: &Buffer, _output: &mut Buffer) -> bool {
        todo!()
    }

    /// Returns `true` if a transition is in progress.
    #[must_use]
    pub fn is_transitioning(&self) -> bool {
        todo!()
    }

    /// Returns a reference to the current scene (idle) or the target scene (transitioning).
    #[must_use]
    pub fn current_scene(&self) -> Option<&Scene> {
        todo!()
    }

    /// Consumes the current state and returns the scene and owner, if idle.
    ///
    /// Returns `None` if the manager has no scene or is mid-transition.
    #[must_use]
    pub fn take(&mut self) -> Option<(Scene, Owner)> {
        todo!()
    }
}

impl Default for TransitionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::CameraConfig;
    use crate::ir::SceneIr;
    use crate::node::{NodeKind, SceneNode, Transform};
    use happyterminals_core::create_root;
    use ratatui_core::layout::Rect;

    /// Create a minimal valid Scene + Owner for testing.
    fn test_scene() -> (Scene, Owner) {
        let (scene, owner) = create_root(|| {
            let ir = SceneIr::new(vec![SceneNode {
                id: crate::node::NodeId::next(),
                kind: NodeKind::Group,
                transform: Transform::default(),
                props: Default::default(),
                children: vec![],
                pipeline: None,
            }]);
            Scene::new(ir, CameraConfig::default(), None).expect("valid test scene")
        });
        (scene, owner)
    }

    #[test]
    fn new_manager_starts_idle_with_no_scene() {
        let mgr = TransitionManager::new();
        assert!(!mgr.is_transitioning());
        assert!(mgr.current_scene().is_none());
    }

    #[test]
    fn set_scene_puts_manager_in_idle() {
        let mut mgr = TransitionManager::new();
        let (scene, owner) = test_scene();
        mgr.set_scene(scene, owner);
        assert!(!mgr.is_transitioning());
        assert!(mgr.current_scene().is_some());
    }

    #[test]
    fn transition_to_moves_to_transitioning() {
        let mut mgr = TransitionManager::new();
        let (s1, o1) = test_scene();
        mgr.set_scene(s1, o1);

        let (s2, o2) = test_scene();
        let result = mgr.transition_to(s2, o2, "dissolve", Duration::from_secs(1));
        assert!(result.is_ok());
        assert!(mgr.is_transitioning());
    }

    #[test]
    fn transition_to_unknown_effect_returns_err() {
        let mut mgr = TransitionManager::new();
        let (s1, o1) = test_scene();
        mgr.set_scene(s1, o1);

        let (s2, o2) = test_scene();
        let result = mgr.transition_to(s2, o2, "nonexistent", Duration::from_secs(1));
        assert!(result.is_err());
    }

    #[test]
    fn tick_advances_progress() {
        let mut mgr = TransitionManager::new();
        let (s1, o1) = test_scene();
        mgr.set_scene(s1, o1);

        let (s2, o2) = test_scene();
        mgr.transition_to(s2, o2, "dissolve", Duration::from_secs(2))
            .unwrap();

        mgr.tick(Duration::from_secs(1));
        // Still transitioning (progress = 0.5)
        assert!(mgr.is_transitioning());
    }

    #[test]
    fn tick_completes_transition_and_disposes_owner() {
        let mut mgr = TransitionManager::new();
        let (s1, o1) = test_scene();
        mgr.set_scene(s1, o1);

        let (s2, o2) = test_scene();
        mgr.transition_to(s2, o2, "dissolve", Duration::from_secs(1))
            .unwrap();

        // Tick past the duration
        mgr.tick(Duration::from_millis(1500));
        assert!(!mgr.is_transitioning());
        assert!(mgr.current_scene().is_some());
    }

    #[test]
    fn register_adds_custom_effect() {
        let mut mgr = TransitionManager::new();
        mgr.register("custom", Dissolve);

        let (s1, o1) = test_scene();
        mgr.set_scene(s1, o1);

        let (s2, o2) = test_scene();
        let result = mgr.transition_to(s2, o2, "custom", Duration::from_secs(1));
        assert!(result.is_ok());
    }

    #[test]
    fn default_registry_has_three_effects() {
        let mut mgr = TransitionManager::new();
        let (s1, o1) = test_scene();
        mgr.set_scene(s1, o1);

        for name in &["dissolve", "slide-left", "fade-to-black"] {
            let (s2, o2) = test_scene();
            let result = mgr.transition_to(s2, o2, name, Duration::from_millis(100));
            assert!(result.is_ok(), "effect '{name}' should be registered");
            // Complete the transition so we're back to idle for the next one
            mgr.tick(Duration::from_millis(200));
        }
    }

    #[test]
    fn blend_during_idle_returns_false() {
        let mut mgr = TransitionManager::new();
        let (s1, o1) = test_scene();
        mgr.set_scene(s1, o1);

        let buf_a = Buffer::empty(Rect::new(0, 0, 4, 2));
        let buf_b = Buffer::empty(Rect::new(0, 0, 4, 2));
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        assert!(!mgr.blend(&buf_a, &buf_b, &mut out));
    }

    #[test]
    fn blend_during_transitioning_returns_true() {
        let mut mgr = TransitionManager::new();
        let (s1, o1) = test_scene();
        mgr.set_scene(s1, o1);

        let (s2, o2) = test_scene();
        mgr.transition_to(s2, o2, "dissolve", Duration::from_secs(1))
            .unwrap();
        mgr.tick(Duration::from_millis(500));

        let buf_a = Buffer::empty(Rect::new(0, 0, 4, 2));
        let buf_b = Buffer::empty(Rect::new(0, 0, 4, 2));
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        assert!(mgr.blend(&buf_a, &buf_b, &mut out));
    }
}
