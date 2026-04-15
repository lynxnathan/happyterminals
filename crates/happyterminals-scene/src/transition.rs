//! [`TransitionManager`] -- scaffold for scene-to-scene transitions.
//!
//! Full transition effects (crossfade, slide, etc.) are deferred to Phase 3.1.
//! This scaffold handles the critical Owner disposal pattern: when transitioning
//! from scene A to scene B, the old Owner must be disposed to clean up all
//! signals, effects, and memos from scene A.
//!
//! # Disposal timing invariant
//!
//! Disposal happens synchronously in [`TransitionManager::set_scene`], which is
//! safe because it is called between frames (not mid-frame). The runtime must
//! ensure `set_scene` is never called while an Effect from the old scene is
//! executing.

use happyterminals_core::Owner;

use crate::scene::Scene;

/// Manages the current scene and handles Owner disposal on transition.
pub struct TransitionManager {
    current: Option<(Scene, Owner)>,
}

impl TransitionManager {
    /// Creates a new transition manager with no current scene.
    #[must_use]
    pub fn new() -> Self {
        Self { current: None }
    }

    /// Sets a new scene, disposing the old Owner if one exists.
    ///
    /// The old [`Owner`] is disposed synchronously, running all `on_cleanup`
    /// callbacks and dropping all signals, memos, and effects from the
    /// previous scene's reactive scope.
    pub fn set_scene(&mut self, scene: Scene, owner: Owner) {
        if let Some((_, old_owner)) = self.current.take() {
            old_owner.dispose();
        }
        self.current = Some((scene, owner));
    }

    /// Returns a reference to the current scene, if any.
    #[must_use]
    pub fn current(&self) -> Option<&Scene> {
        self.current.as_ref().map(|(s, _)| s)
    }

    /// Consumes the manager and returns the current scene and owner, if any.
    #[must_use]
    pub fn take(&mut self) -> Option<(Scene, Owner)> {
        self.current.take()
    }
}

impl Default for TransitionManager {
    fn default() -> Self {
        Self::new()
    }
}
