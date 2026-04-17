//! Snow-over-bunny — physics-y particles interacting with a loaded 3D mesh.
//!
//! White snowflakes fall over the bunny mesh with gravity and spread, fading
//! to light blue with age. Z-buffer occlusion hides particles behind the
//! mesh surface. The particle emitter uses a fixed-capacity pool so NO
//! allocation happens per frame after the initial warmup — a guarantee
//! verified by the alloc-counting benchmark shipped in Phase 2.4.
//!
//! Features exercised:
//! - ParticleEmitter with pool-based, zero-per-frame-allocation update loop
//! - Renderer::draw + Renderer::draw_particles with shared z-buffer
//! - OBJ mesh loading (bunny.obj)
//! - InputMap orbit/zoom controls + custom Space/R bindings
//!
//! Controls:
//! - Left-drag: orbit
//! - Scroll: zoom
//! - Space: toggle pause
//! - R: reset emitter
//! - Q or Ctrl-C: quit
//!
//! Run from the workspace root:
//!
//!     cargo run --example particles -p happyterminals

use happyterminals::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

const BUNNY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/models/bunny.obj"
);
const ZOOM_SENSITIVITY: f32 = 0.5;
const FOV: f32 = std::f32::consts::FRAC_PI_4;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (bunny, _stats) = load_obj(BUNNY_PATH)?;
    let (center, radius) = bunny.bounding_sphere();

    let mut emitter = ParticleEmitter::new(500);
    emitter.origin = center + glam::Vec3::Y * radius * 1.5;
    emitter.spread = glam::Vec3::new(radius, radius * 0.3, radius);
    emitter.gravity = glam::Vec3::new(0.0, -2.0, 0.0);
    emitter.spawn_rate = 50.0;
    emitter.life_range = (3.0, 5.0);
    emitter.color_start = Color::White;
    emitter.color_end = Color::Rgb(180, 200, 255);

    let mut rng = SmallRng::from_rng(&mut rand::rng());
    let mut renderer = Renderer::new();
    let shading = ShadingRamp::default();

    let mut camera = OrbitCamera {
        elevation: 0.4,
        distance: radius * 2.5,
        target: center,
        ..OrbitCamera::default()
    };

    let mut input_map = InputMap::new();
    register_default_actions(&mut input_map);
    input_map.register_action("particle_toggle", ActionValueType::Bool);
    input_map.register_action("particle_reset", ActionValueType::Bool);

    let mut ctx = default_viewer_context();
    ctx.bind("particle_toggle", Binding::Key(KeyCode::Char(' ')), vec![]);
    ctx.bind("particle_reset", Binding::Key(KeyCode::Char('r')), vec![]);
    input_map.push_context(ctx);

    run_with_input(
        move |grid, _input_signals, imap| {
            let vw = f32::from(grid.area.width.max(1));
            let vh = f32::from(grid.area.height.max(1));
            let world_h = 2.0 * camera.distance * (FOV / 2.0).tan();
            let orbit_per_cell = std::f32::consts::PI / vw;

            if let Some(sig) = imap.action_axis2d("orbit") {
                let d = sig.untracked();
                camera.azimuth -= d.x * orbit_per_cell;
                camera.elevation += d.y * orbit_per_cell;
            }
            if let Some(sig) = imap.action_axis1d("zoom") {
                let d = sig.untracked();
                camera.distance = (camera.distance - d * ZOOM_SENSITIVITY).max(0.5);
            }
            if let Some(sig) = imap.action_axis2d("pan") {
                let d = sig.untracked();
                let right = glam::Vec3::new(camera.azimuth.cos(), 0.0, -camera.azimuth.sin());
                let pan_x = (world_h * (vw / vh)) / vw;
                let pan_y = world_h / vh;
                camera.target -= right * d.x * pan_x + glam::Vec3::Y * (-d.y) * pan_y;
            }

            if let Some(s) = imap.action_state("particle_toggle") {
                if s.untracked() == ActionState::JustPressed { emitter.toggle_pause(); }
            }
            if let Some(s) = imap.action_state("particle_reset") {
                if s.untracked() == ActionState::JustPressed { emitter.reset(); }
            }

            let dt = 1.0 / 30.0_f32;
            emitter.update(dt, &mut rng);

            let projection = Projection {
                viewport_w: grid.area.width,
                viewport_h: grid.area.height,
                ..Projection::default()
            };

            renderer.draw(grid, &bunny, &camera, &projection, &shading);
            renderer.draw_particles(grid, &emitter, &camera, &projection, &shading);

            let status = format!(
                " Particles: {} | {} | Space=toggle R=reset Q=quit ",
                emitter.alive_count(),
                if emitter.is_paused() { "PAUSED" } else { "PLAYING" },
            );
            grid.put_str(0, 0, &status, Style::default());
        },
        FrameSpec {
            title: Some("happyterminals - Snow Particles".into()),
            ..FrameSpec::default()
        },
        input_map,
    )
    .await
}
