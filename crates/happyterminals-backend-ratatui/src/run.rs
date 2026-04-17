//! The run loop — entry point for the happyterminals backend.
//!
//! [`run`] drives a `tokio::select!` loop between a frame ticker and
//! `crossterm::EventStream`. Input events are propagated into [`InputSignals`]
//! so that the render callback (and any scene code) can observe them as
//! reactive signal reads.

use std::io;
use std::ops::Deref;
use std::time::Duration;

use crossterm::event::EventStream;
use crossterm::terminal::SetTitle;
use futures::StreamExt;
use ratatui::Terminal;
use ratatui_crossterm::CrosstermBackend;
use tokio::time::{interval, MissedTickBehavior};

use happyterminals_core::grid::Grid;
use happyterminals_core::Rect;
use happyterminals_input::action::ActionState;
use happyterminals_input::InputMap;
use happyterminals_renderer::{Cube, Mesh, Projection, Renderer, ShadingRamp};
use happyterminals_scene::node::{NodeKind, SceneNode};
use happyterminals_scene::transition::TransitionManager;
use happyterminals_scene::{CameraConfig, Scene, SceneIr};

use crate::color::{detect_color_mode_from_real_env, downsample};
use crate::event::{is_quit_event, map_event, InputEvent, InputSignals};
use crate::frame_spec::FrameSpec;
use crate::guard::{install_panic_hook, TerminalGuard};

/// Runs the terminal event loop.
///
/// Creates a [`TerminalGuard`] on entry (RAII — terminal is restored on drop,
/// including panics). Drives a `tokio::select!` loop between:
///
/// - **Frame tick:** calls `render_fn`, copies the [`Grid`] into ratatui's
///   frame buffer, applies flush-time color downsampling, and lets ratatui
///   diff + flush only changed cells.
/// - **Event stream:** maps crossterm events into [`InputEvent`] and writes
///   them into [`InputSignals`] so the render callback can observe them.
///
/// Ctrl+C breaks the loop. The guard drops on return, restoring the terminal.
///
/// # Errors
///
/// Returns an error if terminal acquisition fails or an I/O error occurs
/// during rendering.
pub async fn run<F>(mut render_fn: F, spec: FrameSpec) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&mut Grid, &InputSignals),
{
    install_panic_hook();

    // Detect once at entry. Per CONTEXT + RESEARCH: no per-frame env reads.
    let color_mode = detect_color_mode_from_real_env(spec.color_mode);
    tracing::debug!(?color_mode, "color-mode detected");

    let _guard = TerminalGuard::acquire_with_color_mode(color_mode)?;

    // Best-effort window title
    if let Some(ref title) = spec.title {
        let _ = crossterm::execute!(io::stdout(), SetTitle(title.as_str()));
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut events = EventStream::new();

    let mut tick = interval(spec.frame_duration());
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let (w, h) = crossterm::terminal::size()?;
    let input_signals = InputSignals::new(w, h);
    let mut grid = Grid::new(Rect::new(0, 0, w, h));
    let mut resize_pending = false;

    loop {
        tokio::select! {
            _ = tick.tick() => {
                let _span = tracing::trace_span!("frame").entered();
                terminal.draw(|frame| {
                    grid.resize(frame.area());
                    if resize_pending {
                        resize_pending = false;
                        let mut out = grid.deref().clone();
                        downsample(&mut out, color_mode);
                        *frame.buffer_mut() = out;
                        return;
                    }
                    render_fn(&mut grid, &input_signals);
                    let mut out = grid.deref().clone();
                    downsample(&mut out, color_mode);
                    *frame.buffer_mut() = out;
                })?;
            }
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(ev)) => {
                        if let Some(input) = map_event(&ev) {
                            if is_quit_event(&input) {
                                break;
                            }
                            match &input {
                                InputEvent::Key { .. } => {
                                    input_signals.last_key.set(Some(input));
                                }
                                InputEvent::Mouse { .. } => {
                                    input_signals.last_mouse.set(Some(input));
                                }
                                InputEvent::Resize { width, height } => {
                                    input_signals.terminal_size.set((*width, *height));
                                    resize_pending = true;
                                }
                                InputEvent::FocusGained => {
                                    input_signals.focused.set(true);
                                }
                                InputEvent::FocusLost => {
                                    input_signals.focused.set(false);
                                }
                            }
                        }
                    }
                    Some(Err(_)) | None => break,
                }
            }
        }
    }

    Ok(())
    // _guard drops here, restoring terminal
}

/// Runs the terminal event loop with [`InputMap`] dispatch.
///
/// Like [`run`], creates a [`TerminalGuard`] on entry and drives a
/// `tokio::select!` loop. In addition to legacy [`InputSignals`], events are
/// dispatched through the provided [`InputMap`] so the render callback can
/// read action signals (orbit, zoom, pan, etc.).
///
/// The render callback receives `(&mut Grid, &InputSignals, &InputMap)`.
///
/// # Errors
///
/// Returns an error if terminal acquisition fails or an I/O error occurs
/// during rendering.
pub async fn run_with_input<F>(
    mut render_fn: F,
    spec: FrameSpec,
    mut input_map: InputMap,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&mut Grid, &InputSignals, &InputMap),
{
    install_panic_hook();

    let color_mode = detect_color_mode_from_real_env(spec.color_mode);
    tracing::debug!(?color_mode, "color-mode detected");

    let _guard = TerminalGuard::acquire_with_color_mode(color_mode)?;

    if let Some(ref title) = spec.title {
        let _ = crossterm::execute!(io::stdout(), SetTitle(title.as_str()));
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut events = EventStream::new();

    let mut tick = interval(spec.frame_duration());
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let (w, h) = crossterm::terminal::size()?;
    let input_signals = InputSignals::new(w, h);
    let mut grid = Grid::new(Rect::new(0, 0, w, h));
    let dt = spec.frame_duration();
    let mut resize_pending = false;

    loop {
        tokio::select! {
            _ = tick.tick() => {
                let _span = tracing::trace_span!("frame").entered();
                terminal.draw(|frame| {
                    grid.resize(frame.area());
                    if resize_pending {
                        resize_pending = false;
                        let mut out = grid.deref().clone();
                        downsample(&mut out, color_mode);
                        *frame.buffer_mut() = out;
                        return;
                    }
                    render_fn(&mut grid, &input_signals, &input_map);
                    let mut out = grid.deref().clone();
                    downsample(&mut out, color_mode);
                    *frame.buffer_mut() = out;
                })?;
                // tick_update AFTER render so callback sees JustPressed
                // before it transitions to Held. reset_axes zeros deltas.
                input_map.tick_update(dt);
                input_map.reset_axes();
            }
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(ev)) => {
                        // Dispatch through InputMap FIRST (sees raw crossterm event)
                        input_map.dispatch(&ev);

                        // Check InputMap quit action
                        if let Some(quit_sig) = input_map.action_state("quit") {
                            if quit_sig.untracked() == ActionState::JustPressed {
                                break;
                            }
                        }

                        // Legacy signal path (backward compat)
                        if let Some(input) = map_event(&ev) {
                            if is_quit_event(&input) {
                                break;
                            }
                            match &input {
                                InputEvent::Key { .. } => {
                                    input_signals.last_key.set(Some(input));
                                }
                                InputEvent::Mouse { .. } => {
                                    input_signals.last_mouse.set(Some(input));
                                }
                                InputEvent::Resize { width, height } => {
                                    input_signals.terminal_size.set((*width, *height));
                                    resize_pending = true;
                                }
                                InputEvent::FocusGained => {
                                    input_signals.focused.set(true);
                                }
                                InputEvent::FocusLost => {
                                    input_signals.focused.set(false);
                                }
                            }
                        }
                    }
                    Some(Err(_)) | None => break,
                }
            }
        }
    }

    Ok(())
}

/// Runs the terminal event loop driven by a [`Scene`].
///
/// Like [`run`], creates a [`TerminalGuard`] on entry and drives a
/// `tokio::select!` loop. Instead of a user-provided render closure, the
/// scene IR tree is walked each frame: Cube nodes are rendered via
/// [`Renderer::draw`], and the scene-level pipeline (if any) is applied.
///
/// `tick_fn` is called at the start of every frame tick with the frame
/// duration and input signals, allowing the caller to update reactive
/// signals (e.g. rotation) before the scene is rendered.
///
/// # Errors
///
/// Returns an error if terminal acquisition fails or an I/O error occurs
/// during rendering.
pub async fn run_scene<F>(
    scene: Scene,
    spec: FrameSpec,
    mut tick_fn: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(Duration, &InputSignals),
{
    install_panic_hook();

    // Detect once at entry. Per CONTEXT + RESEARCH: no per-frame env reads.
    let color_mode = detect_color_mode_from_real_env(spec.color_mode);
    tracing::debug!(?color_mode, "color-mode detected");

    let _guard = TerminalGuard::acquire_with_color_mode(color_mode)?;

    // Best-effort window title
    if let Some(ref title) = spec.title {
        let _ = crossterm::execute!(io::stdout(), SetTitle(title.as_str()));
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut events = EventStream::new();

    let mut tick = interval(spec.frame_duration());
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let (w, h) = crossterm::terminal::size()?;
    let input_signals = InputSignals::new(w, h);
    let mut grid = Grid::new(Rect::new(0, 0, w, h));

    // Extract scene parts for mutable access
    let (ir, camera_config, mut pipeline) = scene.into_parts();
    let mut camera_config = camera_config;
    let mut renderer = Renderer::new();
    let shading = ShadingRamp::default();
    // Build the cube mesh ONCE (heap-allocates) so the per-frame hot path
    // borrows it instead of rebuilding. Preserves REND-09 zero-alloc
    // discipline for `NodeKind::Cube` nodes.
    let cube_mesh = Cube::mesh();
    let dt = spec.frame_duration();
    let mut resize_pending = false;

    loop {
        tokio::select! {
            _ = tick.tick() => {
                let _span = tracing::trace_span!("frame").entered();
                if !resize_pending {
                    tick_fn(dt, &input_signals);
                }

                terminal.draw(|frame| {
                    grid.resize(frame.area());
                    if resize_pending {
                        resize_pending = false;
                        let mut out = grid.deref().clone();
                        downsample(&mut out, color_mode);
                        *frame.buffer_mut() = out;
                        return;
                    }
                    let projection = Projection {
                        viewport_w: grid.area.width,
                        viewport_h: grid.area.height,
                        ..Projection::default()
                    };

                    walk_and_render(&ir, &mut grid, &mut renderer, &mut camera_config, &projection, &shading, &cube_mesh);

                    if let Some(ref mut pipe) = pipeline {
                        pipe.run_frame(&mut grid, dt);
                    }

                    let mut out = grid.deref().clone();
                    downsample(&mut out, color_mode);
                    *frame.buffer_mut() = out;
                })?;
            }
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(ev)) => {
                        if let Some(input) = map_event(&ev) {
                            if is_quit_event(&input) {
                                break;
                            }
                            match &input {
                                InputEvent::Key { .. } => {
                                    input_signals.last_key.set(Some(input));
                                }
                                InputEvent::Mouse { .. } => {
                                    input_signals.last_mouse.set(Some(input));
                                }
                                InputEvent::Resize { width, height } => {
                                    input_signals.terminal_size.set((*width, *height));
                                    resize_pending = true;
                                }
                                InputEvent::FocusGained => {
                                    input_signals.focused.set(true);
                                }
                                InputEvent::FocusLost => {
                                    input_signals.focused.set(false);
                                }
                            }
                        }
                    }
                    Some(Err(_)) | None => break,
                }
            }
        }
    }

    Ok(())
}

/// Runs the terminal event loop with a [`TransitionManager`] driving
/// scene-to-scene transitions.
///
/// Like [`run_with_input`], creates a [`TerminalGuard`] on entry and drives a
/// `tokio::select!` loop. Events are dispatched through the provided
/// [`InputMap`]. On each frame tick:
///
/// 1. `tick_fn` is called, giving the caller access to the `TransitionManager`
///    to trigger transitions programmatically.
/// 2. `TransitionManager::tick(dt)` advances any in-progress transition.
/// 3. If transitioning, both scenes are rendered into separate buffers and
///    blended via the active effect. Otherwise the current scene is rendered
///    normally.
///
/// # Errors
///
/// Returns an error if terminal acquisition fails or an I/O error occurs
/// during rendering.
pub async fn run_scenes<F>(
    mut transition_manager: TransitionManager,
    spec: FrameSpec,
    mut input_map: InputMap,
    mut tick_fn: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(Duration, &InputSignals, &mut TransitionManager),
{
    install_panic_hook();

    let color_mode = detect_color_mode_from_real_env(spec.color_mode);
    tracing::debug!(?color_mode, "color-mode detected");

    let _guard = TerminalGuard::acquire_with_color_mode(color_mode)?;

    if let Some(ref title) = spec.title {
        let _ = crossterm::execute!(io::stdout(), SetTitle(title.as_str()));
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut events = EventStream::new();

    let mut tick = interval(spec.frame_duration());
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let (w, h) = crossterm::terminal::size()?;
    let input_signals = InputSignals::new(w, h);
    let mut grid = Grid::new(Rect::new(0, 0, w, h));
    let dt = spec.frame_duration();
    let mut resize_pending = false;

    // Pre-allocate two renderers to avoid z-buffer interference (RESEARCH Pitfall 5)
    let mut renderer_a = Renderer::new();
    let mut renderer_b = Renderer::new();
    let cube_mesh = Cube::mesh();
    let shading = ShadingRamp::default();

    loop {
        tokio::select! {
            _ = tick.tick() => {
                let _span = tracing::trace_span!("frame").entered();
                if !resize_pending {
                    tick_fn(dt, &input_signals, &mut transition_manager);
                    transition_manager.tick(dt);
                }

                terminal.draw(|frame| {
                    grid.resize(frame.area());
                    if resize_pending {
                        resize_pending = false;
                        let mut out = grid.deref().clone();
                        downsample(&mut out, color_mode);
                        *frame.buffer_mut() = out;
                        return;
                    }

                    let mut render_ctx = RenderCtx {
                        renderer_a: &mut renderer_a, renderer_b: &mut renderer_b,
                        cube_mesh: &cube_mesh, shading: &shading, dt, color_mode,
                    };
                    render_transition_frame(
                        &transition_manager, &mut grid, &mut render_ctx, frame,
                    );
                })?;
                input_map.tick_update(dt);
                input_map.reset_axes();
            }
            maybe_event = events.next() => {
                let should_break = dispatch_input_event(
                    maybe_event.as_ref(), &mut input_map, &input_signals, &mut resize_pending,
                );
                if should_break { break; }
            }
        }
    }

    Ok(())
}

/// Shared render resources for [`render_transition_frame`].
struct RenderCtx<'a> {
    renderer_a: &'a mut Renderer,
    renderer_b: &'a mut Renderer,
    cube_mesh: &'a Mesh,
    shading: &'a ShadingRamp<'a>,
    dt: Duration,
    color_mode: crate::color::ColorMode,
}

/// Render a single frame for [`run_scenes`], handling both idle and transition states.
fn render_transition_frame(
    transition_manager: &TransitionManager,
    grid: &mut Grid,
    ctx: &mut RenderCtx<'_>,
    frame: &mut ratatui::Frame<'_>,
) {
    let projection = Projection {
        viewport_w: grid.area.width,
        viewport_h: grid.area.height,
        ..Projection::default()
    };

    if transition_manager.is_transitioning() {
        let (from_scene, to_scene) = transition_manager.scenes_for_render();
        if let (Some(from), Some(to)) = (from_scene, to_scene) {
            let mut grid_a = Grid::new(grid.area);
            render_scene_to_grid(from, &mut grid_a, ctx.renderer_a, &projection, ctx.shading, ctx.cube_mesh, ctx.dt);
            let buf_a = grid_a.deref().clone();

            let mut grid_b = Grid::new(grid.area);
            render_scene_to_grid(to, &mut grid_b, ctx.renderer_b, &projection, ctx.shading, ctx.cube_mesh, ctx.dt);
            let buf_b = grid_b.deref().clone();

            let mut out = ratatui::buffer::Buffer::empty(grid.area);
            transition_manager.blend(&buf_a, &buf_b, &mut out);
            downsample(&mut out, ctx.color_mode);
            *frame.buffer_mut() = out;
        }
    } else {
        let (scene, _) = transition_manager.scenes_for_render();
        if let Some(scene) = scene {
            render_scene_to_grid(scene, grid, ctx.renderer_a, &projection, ctx.shading, ctx.cube_mesh, ctx.dt);
        }

        let mut out = (*grid).deref().clone();
        downsample(&mut out, ctx.color_mode);
        *frame.buffer_mut() = out;
    }
}

/// Dispatch an input event through [`InputMap`] and legacy signals. Returns
/// `true` if the loop should break (quit event detected).
fn dispatch_input_event(
    maybe_event: Option<&Result<crossterm::event::Event, std::io::Error>>,
    input_map: &mut InputMap,
    input_signals: &InputSignals,
    resize_pending: &mut bool,
) -> bool {
    match maybe_event {
        Some(Ok(ev)) => {
            input_map.dispatch(ev);

            if let Some(quit_sig) = input_map.action_state("quit") {
                if quit_sig.untracked() == ActionState::JustPressed {
                    return true;
                }
            }

            if let Some(input) = map_event(ev) {
                if is_quit_event(&input) {
                    return true;
                }
                match &input {
                    InputEvent::Key { .. } => {
                        input_signals.last_key.set(Some(input));
                    }
                    InputEvent::Mouse { .. } => {
                        input_signals.last_mouse.set(Some(input));
                    }
                    InputEvent::Resize { width, height } => {
                        input_signals.terminal_size.set((*width, *height));
                        *resize_pending = true;
                    }
                    InputEvent::FocusGained => {
                        input_signals.focused.set(true);
                    }
                    InputEvent::FocusLost => {
                        input_signals.focused.set(false);
                    }
                }
            }
            false
        }
        _ => true, // Some(Err(_)) | None => break
    }
}

/// Render a scene into a grid (used by [`run_scenes`] for both single and
/// two-buffer rendering). Reads scene camera/IR/pipeline immutably.
fn render_scene_to_grid(
    scene: &Scene,
    grid: &mut Grid,
    renderer: &mut Renderer,
    projection: &Projection,
    shading: &ShadingRamp<'_>,
    cube_mesh: &Mesh,
    _dt: Duration,
) {
    let ir = scene.ir();
    let camera_config = scene.camera();
    for node in ir.nodes() {
        render_node_immutable(node, grid, renderer, camera_config, projection, shading, cube_mesh);
    }
}

/// Recursively render a single scene node with an immutable camera reference.
///
/// Used by [`render_scene_to_grid`] during transitions where we cannot mutably
/// borrow the camera (scenes are borrowed from `TransitionManager`).
fn render_node_immutable(
    node: &SceneNode,
    grid: &mut Grid,
    renderer: &mut Renderer,
    camera_config: &CameraConfig,
    projection: &Projection,
    shading: &ShadingRamp<'_>,
    cube_mesh: &Mesh,
) {
    match &node.kind {
        NodeKind::Cube => {
            let cam = camera_config.as_camera();
            renderer.draw(grid, cube_mesh, cam, projection, shading);
        }
        NodeKind::Layer { .. } | NodeKind::Group => {
            for child in &node.children {
                render_node_immutable(child, grid, renderer, camera_config, projection, shading, cube_mesh);
            }
        }
        NodeKind::Custom(_) => {}
    }
}

/// Walk all root nodes in the scene IR and render each.
fn walk_and_render(
    ir: &SceneIr,
    grid: &mut Grid,
    renderer: &mut Renderer,
    camera_config: &mut CameraConfig,
    projection: &Projection,
    shading: &ShadingRamp<'_>,
    cube_mesh: &Mesh,
) {
    for node in ir.nodes() {
        render_node(node, grid, renderer, camera_config, projection, shading, cube_mesh);
    }
}

/// Recursively render a single scene node.
fn render_node(
    node: &SceneNode,
    grid: &mut Grid,
    renderer: &mut Renderer,
    camera_config: &mut CameraConfig,
    projection: &Projection,
    shading: &ShadingRamp<'_>,
    cube_mesh: &Mesh,
) {
    match &node.kind {
        NodeKind::Cube => {
            // Read reactive rotation prop without subscribing (REACT-07)
            if let Some(prop) = node.props.get("rotation") {
                if let Some(angle) = prop.read_untracked::<f32>() {
                    if let Some(cam) = camera_config.as_orbit_mut() {
                        cam.azimuth = angle;
                    }
                }
            }
            let cam = camera_config.as_camera();
            renderer.draw(grid, cube_mesh, cam, projection, shading);
        }
        NodeKind::Layer { .. } | NodeKind::Group => {
            for child in &node.children {
                render_node(child, grid, renderer, camera_config, projection, shading, cube_mesh);
            }
        }
        NodeKind::Custom(_) => {}
    }
}
