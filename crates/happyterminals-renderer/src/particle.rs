//! Pool-based particle system with zero per-frame allocation.
//!
//! [`Particle`] is a `Copy` struct representing a single point particle with
//! position, velocity, lifetime, and color-over-time. [`ParticleEmitter`] owns
//! a fixed-size pool of particles and provides spawn/update logic using a
//! fractional accumulator pattern.
//!
//! # Zero-allocation guarantee
//!
//! After [`ParticleEmitter::new`] allocates the pool, no subsequent call to
//! [`update`](ParticleEmitter::update) performs heap allocation. Dead particles
//! are recycled in-place by scanning for the first `alive == false` slot.

use glam::Vec3;
use rand::Rng;
use ratatui_core::style::Color;

/// Maximum number of particles spawned per single `update()` call.
///
/// Caps burst spawning to prevent frame-time spikes when `spawn_rate * dt`
/// is large (e.g., after a long frame or very high spawn rate).
const MAX_SPAWNS_PER_FRAME: u32 = 10;

/// A single point particle with position, velocity, lifetime, and color.
///
/// All fields are `Copy` — no heap allocation per particle.
#[derive(Debug, Clone, Copy)]
pub struct Particle {
    /// World-space position.
    pub position: Vec3,
    /// World-space velocity (units per second).
    pub velocity: Vec3,
    /// Remaining lifetime in seconds. When <= 0, the particle is dead.
    pub life: f32,
    /// Original lifetime at spawn (used for normalized age computation).
    pub max_life: f32,
    /// Color at birth (young particles).
    pub color_start: Color,
    /// Color at death (old particles).
    pub color_end: Color,
    /// Whether this particle is alive and should be updated/rendered.
    pub alive: bool,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            life: 0.0,
            max_life: 1.0,
            color_start: Color::White,
            color_end: Color::White,
            alive: false,
        }
    }
}

/// Linearly interpolate between two colors.
///
/// For `Rgb` variants, interpolates each channel independently.
/// For non-`Rgb` variants, returns `start` unchanged (terminal indexed
/// colors have no meaningful interpolation).
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn lerp_color(start: Color, end: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    match (start, end) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let r = (f32::from(r1) + (f32::from(r2) - f32::from(r1)) * t) as u8;
            let g = (f32::from(g1) + (f32::from(g2) - f32::from(g1)) * t) as u8;
            let b = (f32::from(b1) + (f32::from(b2) - f32::from(b1)) * t) as u8;
            Color::Rgb(r, g, b)
        }
        _ => start,
    }
}

/// Pool-based particle emitter with zero per-frame allocation.
///
/// The pool is allocated once at construction. Subsequent calls to
/// [`update`](Self::update) recycle dead slots without heap allocation.
pub struct ParticleEmitter {
    /// Fixed-size particle pool. Never grows after construction.
    pub particles: Vec<Particle>,
    /// Particles spawned per second.
    pub spawn_rate: f32,
    /// Fractional accumulator for frame-rate-independent spawning.
    pub spawn_accumulator: f32,
    /// Gravity applied to all alive particles (units/s^2).
    pub gravity: Vec3,
    /// World-space origin of the spawn volume.
    pub origin: Vec3,
    /// Half-extents of the spawn volume (position randomized within).
    pub spread: Vec3,
    /// Min and max lifetime for newly spawned particles.
    pub life_range: (f32, f32),
    /// Start color for newly spawned particles.
    pub color_start: Color,
    /// End color for newly spawned particles.
    pub color_end: Color,
    /// Whether the emitter is paused (no updates, no spawning).
    paused: bool,
}

impl ParticleEmitter {
    /// Create a new emitter with a fixed-capacity pool.
    ///
    /// All particles start dead (`alive == false`). The pool never grows
    /// after this allocation.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            particles: vec![Particle::default(); capacity],
            spawn_rate: 50.0,
            spawn_accumulator: 0.0,
            gravity: Vec3::new(0.0, -2.0, 0.0),
            origin: Vec3::ZERO,
            spread: Vec3::ONE,
            life_range: (3.0, 5.0),
            color_start: Color::White,
            color_end: Color::Rgb(180, 200, 255),
            paused: false,
        }
    }

    /// Update all alive particles and spawn new ones.
    ///
    /// 1. Applies gravity to velocity, velocity to position, decrements life.
    /// 2. Marks dead particles (`life <= 0`).
    /// 3. Spawns new particles via the fractional accumulator (capped at
    ///    [`MAX_SPAWNS_PER_FRAME`]).
    ///
    /// Does nothing if paused.
    pub fn update(&mut self, dt: f32, rng: &mut impl Rng) {
        if self.paused {
            return;
        }

        // Update alive particles
        for p in &mut self.particles {
            if !p.alive {
                continue;
            }
            p.velocity += self.gravity * dt;
            p.position += p.velocity * dt;
            p.life -= dt;
            if p.life <= 0.0 {
                p.alive = false;
            }
        }

        // Spawn new particles via accumulator
        self.spawn_accumulator += self.spawn_rate * dt;
        let mut spawned = 0u32;
        while self.spawn_accumulator >= 1.0 && spawned < MAX_SPAWNS_PER_FRAME {
            self.spawn_accumulator -= 1.0;
            self.spawn_one(rng);
            spawned += 1;
        }
    }

    /// Spawn a single particle into the first available dead slot.
    ///
    /// If the pool is full (all alive), silently drops — no allocation.
    fn spawn_one(&mut self, rng: &mut impl Rng) {
        let Some(p) = self.particles.iter_mut().find(|p| !p.alive) else {
            return;
        };

        let life = rng.random_range(self.life_range.0..=self.life_range.1);
        let offset = Vec3::new(
            rng.random_range(-self.spread.x..=self.spread.x),
            rng.random_range(-self.spread.y..=self.spread.y),
            rng.random_range(-self.spread.z..=self.spread.z),
        );

        *p = Particle {
            position: self.origin + offset,
            velocity: Vec3::new(
                rng.random_range(-0.2..=0.2),
                rng.random_range(-0.1..=0.1),
                rng.random_range(-0.2..=0.2),
            ),
            life,
            max_life: life,
            color_start: self.color_start,
            color_end: self.color_end,
            alive: true,
        };
    }

    /// Mark all particles as dead and reset the spawn accumulator.
    pub fn reset(&mut self) {
        for p in &mut self.particles {
            p.alive = false;
        }
        self.spawn_accumulator = 0.0;
    }

    /// Toggle between paused and unpaused.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Returns `true` if the emitter is currently paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Iterate over alive particles only.
    pub fn alive_particles(&self) -> impl Iterator<Item = &Particle> {
        self.particles.iter().filter(|p| p.alive)
    }

    /// Count of currently alive particles.
    #[must_use]
    pub fn alive_count(&self) -> usize {
        self.particles.iter().filter(|p| p.alive).count()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn test_rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    #[test]
    fn particle_is_copy() {
        let p = Particle::default();
        // Use p, then copy it -- this would fail to compile if Particle is not Copy.
        let _ = p.position;
        let p2: Particle = p;
        assert!(!p2.alive);
    }

    #[test]
    fn emitter_new_creates_dead_pool() {
        let emitter = ParticleEmitter::new(100);
        assert_eq!(emitter.particles.len(), 100);
        assert_eq!(emitter.particles.capacity(), 100);
        assert_eq!(emitter.alive_count(), 0);
    }

    #[test]
    fn update_spawns_particles() {
        let mut emitter = ParticleEmitter::new(100);
        emitter.spawn_rate = 100.0;
        let mut rng = test_rng();

        // dt=0.1 -> spawn_rate * dt = 10 -> should spawn up to 10 particles
        emitter.update(0.1, &mut rng);
        assert!(
            emitter.alive_count() > 0,
            "Should have spawned particles, got alive_count=0"
        );
    }

    #[test]
    fn update_100_frames_does_not_grow_capacity() {
        let mut emitter = ParticleEmitter::new(200);
        emitter.spawn_rate = 50.0;
        let mut rng = test_rng();
        let cap_before = emitter.particles.capacity();

        for _ in 0..100 {
            emitter.update(0.016, &mut rng);
        }

        assert_eq!(
            emitter.particles.capacity(),
            cap_before,
            "Pool capacity must not grow after init"
        );
    }

    #[test]
    fn dead_particles_marked_not_alive() {
        let mut emitter = ParticleEmitter::new(50);
        emitter.spawn_rate = 100.0;
        emitter.life_range = (0.01, 0.02); // very short life
        let mut rng = test_rng();

        // Spawn some particles
        emitter.update(0.1, &mut rng);
        assert!(emitter.alive_count() > 0);

        // Run long enough for them all to die
        for _ in 0..100 {
            emitter.update(0.1, &mut rng);
        }
        // After 10 seconds with 0.01-0.02 life, all initial particles are dead.
        // New ones may have spawned, but at least some should have died.
        // (We cannot assert all dead because new ones keep spawning.)
        // Instead, check that particles with negative life are marked dead.
        for p in &emitter.particles {
            if p.life <= 0.0 {
                assert!(!p.alive, "Particle with life <= 0 should be dead");
            }
        }
    }

    #[test]
    fn reset_kills_all_particles() {
        let mut emitter = ParticleEmitter::new(100);
        emitter.spawn_rate = 100.0;
        let mut rng = test_rng();

        emitter.update(0.1, &mut rng);
        assert!(emitter.alive_count() > 0);

        emitter.reset();
        assert_eq!(emitter.alive_count(), 0, "All particles should be dead after reset");
    }

    #[test]
    fn toggle_pause_stops_updates() {
        let mut emitter = ParticleEmitter::new(100);
        emitter.spawn_rate = 100.0;
        let mut rng = test_rng();

        emitter.toggle_pause();
        assert!(emitter.is_paused());

        emitter.update(0.1, &mut rng);
        assert_eq!(emitter.alive_count(), 0, "Paused emitter should not spawn");

        emitter.toggle_pause();
        assert!(!emitter.is_paused());

        emitter.update(0.1, &mut rng);
        assert!(emitter.alive_count() > 0, "Unpaused emitter should spawn");
    }

    #[test]
    fn lerp_color_midpoint() {
        let c = lerp_color(Color::Rgb(0, 0, 0), Color::Rgb(255, 255, 255), 0.5);
        match c {
            Color::Rgb(r, g, b) => {
                assert!(
                    (i16::from(r) - 127).unsigned_abs() <= 1,
                    "R should be ~127, got {r}"
                );
                assert!(
                    (i16::from(g) - 127).unsigned_abs() <= 1,
                    "G should be ~127, got {g}"
                );
                assert!(
                    (i16::from(b) - 127).unsigned_abs() <= 1,
                    "B should be ~127, got {b}"
                );
            }
            other => panic!("Expected Rgb, got {other:?}"),
        }
    }

    #[test]
    fn lerp_color_non_rgb_returns_start() {
        let c = lerp_color(Color::Red, Color::Blue, 0.5);
        assert_eq!(c, Color::Red, "Non-Rgb lerp should return start color");
    }

    #[test]
    fn lerp_color_at_zero_and_one() {
        let start = Color::Rgb(10, 20, 30);
        let end = Color::Rgb(200, 210, 220);

        let at_zero = lerp_color(start, end, 0.0);
        assert_eq!(at_zero, start, "lerp at t=0 should return start");

        let at_one = lerp_color(start, end, 1.0);
        assert_eq!(at_one, end, "lerp at t=1 should return end");
    }
}
