//! [`TransitionManager`] -- scene-to-scene transitions with named effects.
//!
//! The transition manager maintains a state machine ([`TransitionState`]) that
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
use crate::error::SceneError;
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
        let mut registry: HashMap<String, Box<dyn TransitionEffect>> = HashMap::new();
        registry.insert("dissolve".into(), Box::new(Dissolve));
        registry.insert("slide-left".into(), Box::new(SlideLeft));
        registry.insert("fade-to-black".into(), Box::new(FadeToBlack));
        Self {
            state: None,
            registry,
        }
    }

    /// Sets a new scene immediately (no transition). Disposes old owner if present.
    pub fn set_scene(&mut self, scene: Scene, owner: Owner) {
        if let Some(old_state) = self.state.take() {
            match old_state {
                TransitionState::Idle {
                    owner: old_owner, ..
                } => {
                    old_owner.dispose();
                }
                TransitionState::Transitioning {
                    from_owner,
                    to_owner,
                    ..
                } => {
                    from_owner.dispose();
                    to_owner.dispose();
                }
            }
        }
        self.state = Some(TransitionState::Idle { scene, owner });
    }

    /// Registers a named transition effect.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        effect: impl TransitionEffect + 'static,
    ) {
        self.registry.insert(name.into(), Box::new(effect));
    }

    /// Starts a transition to a new scene using the named effect and linear easing.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::UnknownEffect`] if the effect name is not registered.
    /// Returns [`SceneError::NotIdle`] if not in the Idle state.
    pub fn transition_to(
        &mut self,
        scene: Scene,
        owner: Owner,
        effect_name: &str,
        duration: Duration,
    ) -> Result<(), SceneError> {
        self.transition_to_with_easing(scene, owner, effect_name, duration, easing::linear)
    }

    /// Starts a transition with a custom easing function.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::UnknownEffect`] if the effect name is not registered.
    /// Returns [`SceneError::NotIdle`] if not in the Idle state.
    pub fn transition_to_with_easing(
        &mut self,
        scene: Scene,
        owner: Owner,
        effect_name: &str,
        duration: Duration,
        easing_fn: fn(f32) -> f32,
    ) -> Result<(), SceneError> {
        // Validate effect exists
        if !self.registry.contains_key(effect_name) {
            return Err(SceneError::UnknownEffect {
                name: effect_name.to_string(),
            });
        }

        // Must be in Idle state
        let current = self.state.take();
        match current {
            Some(TransitionState::Idle {
                scene: from_scene,
                owner: from_owner,
            }) => {
                self.state = Some(TransitionState::Transitioning {
                    from_scene,
                    from_owner,
                    to_scene: scene,
                    to_owner: owner,
                    effect_name: effect_name.to_string(),
                    elapsed: Duration::ZERO,
                    duration,
                    easing: easing_fn,
                });
                Ok(())
            }
            Some(state @ TransitionState::Transitioning { .. }) => {
                // Put back the state
                self.state = Some(state);
                Err(SceneError::NotIdle {
                    state: "transitioning".to_string(),
                })
            }
            None => Err(SceneError::NotIdle {
                state: "no scene loaded".to_string(),
            }),
        }
    }

    /// Advances the transition by `dt`. If progress reaches 1.0, the old owner
    /// is disposed and the manager returns to Idle with the new scene.
    pub fn tick(&mut self, dt: Duration) {
        let current = self.state.take();
        self.state = match current {
            Some(TransitionState::Transitioning {
                from_scene,
                from_owner,
                to_scene,
                to_owner,
                effect_name,
                elapsed,
                duration,
                easing: easing_fn,
            }) => {
                let new_elapsed = elapsed + dt;
                let raw_progress = new_elapsed.as_secs_f32() / duration.as_secs_f32();
                let progress = (easing_fn)(raw_progress).clamp(0.0, 1.0);

                if progress >= 1.0 {
                    // Transition complete: dispose old owner, move to Idle
                    from_owner.dispose();
                    // from_scene is dropped here (no longer needed)
                    drop(from_scene);
                    Some(TransitionState::Idle {
                        scene: to_scene,
                        owner: to_owner,
                    })
                } else {
                    Some(TransitionState::Transitioning {
                        from_scene,
                        from_owner,
                        to_scene,
                        to_owner,
                        effect_name,
                        elapsed: new_elapsed,
                        duration,
                        easing: easing_fn,
                    })
                }
            }
            other => other, // Idle or None: no-op
        };
    }

    /// Blends the two scene buffers during a transition.
    ///
    /// Returns `true` if a blend was performed (transitioning), `false` if idle.
    pub fn blend(&self, buf_a: &Buffer, buf_b: &Buffer, output: &mut Buffer) -> bool {
        match &self.state {
            Some(TransitionState::Transitioning {
                effect_name,
                elapsed,
                duration,
                easing: easing_fn,
                ..
            }) => {
                let raw_progress = elapsed.as_secs_f32() / duration.as_secs_f32();
                let progress = (easing_fn)(raw_progress).clamp(0.0, 1.0);

                if let Some(effect) = self.registry.get(effect_name.as_str()) {
                    effect.blend(buf_a, buf_b, progress, output);
                }
                true
            }
            _ => false,
        }
    }

    /// Returns `true` if a transition is in progress.
    #[must_use]
    pub fn is_transitioning(&self) -> bool {
        matches!(&self.state, Some(TransitionState::Transitioning { .. }))
    }

    /// Returns a reference to the current scene (idle) or the target scene (transitioning).
    #[must_use]
    pub fn current_scene(&self) -> Option<&Scene> {
        match &self.state {
            Some(TransitionState::Idle { scene, .. }) => Some(scene),
            Some(TransitionState::Transitioning { to_scene, .. }) => Some(to_scene),
            None => None,
        }
    }

    /// Returns a mutable reference to the current scene's camera config.
    ///
    /// In Idle state, returns the active scene's camera.
    /// During a transition, returns the destination scene's camera (the one
    /// the user will be looking at after the transition completes).
    pub fn current_camera_mut(&mut self) -> Option<&mut crate::camera::CameraConfig> {
        match &mut self.state {
            Some(TransitionState::Idle { scene, .. }) => Some(scene.camera_mut()),
            Some(TransitionState::Transitioning { to_scene, .. }) => Some(to_scene.camera_mut()),
            None => None,
        }
    }

    /// Returns references to scene(s) for rendering.
    ///
    /// - If idle: returns `(Some(scene), None)`.
    /// - If transitioning: returns `(Some(from_scene), Some(to_scene))`.
    /// - If no scene loaded: returns `(None, None)`.
    #[must_use]
    pub fn scenes_for_render(&self) -> (Option<&Scene>, Option<&Scene>) {
        match &self.state {
            Some(TransitionState::Idle { scene, .. }) => (Some(scene), None),
            Some(TransitionState::Transitioning {
                from_scene,
                to_scene,
                ..
            }) => (Some(from_scene), Some(to_scene)),
            None => (None, None),
        }
    }

    /// Consumes the current state and returns the scene and owner, if idle.
    ///
    /// Returns `None` if the manager has no scene or is mid-transition.
    #[must_use]
    pub fn take(&mut self) -> Option<(Scene, Owner)> {
        match self.state.take() {
            Some(TransitionState::Idle { scene, owner }) => Some((scene, owner)),
            other => {
                // Put it back if not idle
                self.state = other;
                None
            }
        }
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
