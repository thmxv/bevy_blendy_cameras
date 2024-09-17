//! Camera Controllers and tools for editor like 3D views
//!
//! Inspired by Blender's viewport camera controls and tools and provide
//! the same functionalities:
//! - Pan/Orbit/Zoom camera controls with "Zoom to mouse position" and
//!   "Auto depth".
//! - Fly camera controls with zooming (back, forward), panning (left, right,
//!   down, up) and speed controls.
//! - Set viewpoint: View from left, right, front, back, top and bottom.
//! - Frame entities into view: Allowing to do things like framing the whole
//!   scene, or the selected objects.
//! - Switch between orthographic and perspective camera projection

use bevy::{
    input::{keyboard::KeyCode, mouse::MouseWheel, ButtonInput},
    prelude::*,
    render::camera::{CameraUpdateSystem, RenderTarget},
    transform::TransformSystem,
    window::{CursorGrabMode, PrimaryWindow, WindowRef},
    winit::WinitWindows,
};
#[cfg(feature = "bevy_egui")]
use bevy_egui::EguiSet;
use bevy_mod_raycast::prelude::*;

#[cfg(feature = "bevy_egui")]
pub use crate::egui::EguiWantsFocus;
use crate::{
    fly::fly_camera_controller_system,
    frame::frame_system,
    input::{mouse_key_tracker_system, MouseKeyTracker},
    orbit::orbit_camera_controller_system,
    raycast::{
        add_to_raycast_system, remove_from_raycast_system,
        BlendyCamerasRaycastSet,
    },
    viewpoints::viewpoint_system,
};
pub use crate::{
    fly::FlyCameraController,
    frame::FrameEvent,
    orbit::OrbitCameraController,
    viewpoints::{Viewpoint, ViewpointEvent},
};

#[cfg(feature = "bevy_egui")]
mod egui;
mod fly;
mod frame;
mod input;
mod orbit;
mod raycast;
mod utils;
mod viewpoints;

/// Event to switch between perspective and ortographic camera projections
#[derive(Event)]
pub struct SwitchProjection {
    /// The camera entity for switch to change the view projection
    pub camera_entity: Entity,
}

/// Event to enable the [`OrbitCameraController`] and disable the
/// [`FlyCameraController`] if present
#[derive(Event)]
pub struct SwitchToOrbitController {
    /// The camera entity to switch to pan/orbit/zoom control mode
    pub camera_entity: Entity,
}

/// Event to enable the [`FlyCameraController`] and disable the
/// [`OrbitCameraController`] if present
#[derive(Event)]
pub struct SwitchToFlyController {
    /// The camera entity to switch to fly control mode
    pub camera_entity: Entity,
}

/// Component that contains the saved camera projection (orthographic,
/// perspective) to be switched to when switching camera projection
#[derive(Component)]
pub(crate) struct OtherProjection(Projection);

/// System set to allow ordering
#[derive(Debug, Clone, Copy, SystemSet, PartialEq, Eq, Hash)]
pub enum BlendyCamerasSystemSet {
    /// Check if egui has the focus
    #[cfg(feature = "bevy_egui")]
    CheckEguiWantsFocus,
    /// Process the input and check which camera is active
    ProcessInput,
    /// Handle the [`SwitchProjection`], [`SwitchToOrbitController`],
    /// [`SwitchToFlyController`], [`ViewpointEvent`] and [`FrameEvent`]
    /// events
    HandleEvents,
    /// Handle the [`OrbitCameraController`] and [`FlyCameraController`] only
    /// if egui has not the focus
    Controllers,
}

/// Bevy pluging that contains all the systems necessarty to this crate
pub struct BlendyCamerasPlugin;

impl Plugin for BlendyCamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DeferredRaycastingPlugin::<BlendyCamerasRaycastSet>::default(),
        )
        .init_resource::<ActiveCameraData>()
        .init_resource::<MouseKeyTracker>()
        .add_event::<SwitchProjection>()
        .add_event::<SwitchToOrbitController>()
        .add_event::<SwitchToFlyController>()
        .add_event::<ViewpointEvent>()
        .add_event::<FrameEvent>()
        .add_systems(
            Update,
            (add_to_raycast_system, remove_from_raycast_system),
        )
        .add_systems(
            PostUpdate,
            (
                active_viewport_data_system.run_if(
                    |active_cam: Res<ActiveCameraData>| !active_cam.manual,
                ),
                (mouse_key_tracker_system, wrap_grab_center_cursor_system),
            )
                .chain()
                .in_set(BlendyCamerasSystemSet::ProcessInput),
        )
        .add_systems(
            PostUpdate,
            (
                switch_camera_projection_system,
                (
                    switch_to_fly_camera_controller_system,
                    switch_to_orbit_camera_controller_system,
                )
                    .after(switch_camera_projection_system),
                viewpoint_system,
                frame_system,
            )
                .in_set(BlendyCamerasSystemSet::HandleEvents)
                .after(BlendyCamerasSystemSet::ProcessInput),
        )
        .add_systems(
            PostUpdate,
            (orbit_camera_controller_system, fly_camera_controller_system)
                .in_set(BlendyCamerasSystemSet::Controllers)
                .after(BlendyCamerasSystemSet::HandleEvents)
                .before(CameraUpdateSystem)
                .before(TransformSystem::TransformPropagate),
        );
        #[cfg(feature = "bevy_egui")]
        {
            app.init_resource::<EguiWantsFocus>().add_systems(
                PreUpdate,
                egui::check_egui_wants_focus
                    .in_set(BlendyCamerasSystemSet::CheckEguiWantsFocus)
                    .after(EguiSet::BeginFrame),
            );
        }
    }
}

/// Tracks which `PanOrbitCamera` is active (should handle input events),
/// along with the window and viewport dimensions, which are used for scaling
/// mouse motion.
/// `BlendyCamerasPlugin` manages this resource automatically, in order to
/// support multiple viewports/windows. However, if this doesn't work for you,
/// you can take over and manage it yourself, e.g. when you want to control a
/// camera that is rendering to a texture.
#[derive(Resource, Default, Debug, PartialEq)]
pub struct ActiveCameraData {
    /// ID of the entity with `OrbitCameraController` or `FlyCameraController`
    /// that will handle user input. In other words, this is the camera that
    /// will move when you rotate/pan/zoom.
    pub entity: Option<Entity>,
    /// The viewport size. This is only used to scale the panning mouse motion.
    /// I recommend setting this to the actual render target dimensions (e.g.
    /// the image or viewport), and changing `PanOrbitCamera::pan_sensitivity`
    /// to adjust the sensitivity if required.
    pub viewport_size: Option<Vec2>,
    /// The size of the window. This is only used to scale the orbit mouse
    /// motion. I recommend setting this to actual dimensions of the window
    /// that you want to control the camera from, and changing
    /// `PanOrbitCamera::orbit_sensitivity` to adjust the sensitivity if
    /// required.
    pub window_size: Option<Vec2>,
    /// Indicates to `BevyCamerasPlugin` that it should not update/overwrite
    /// this resource. If you are manually updating this resource you should
    /// set this to `true`. Note that setting this to `true` will effectively
    /// break multiple viewport/window support unless you manually reimplement
    /// it.
    pub manual: bool,
    /// The entity of the window containing the viewport. This is used to grab
    /// or wrap around the cursor while controlling the camera with mouse
    /// movements.
    pub window_entity: Option<Entity>,
}

// TODO: Rename
fn get_window_if_cursor_in_camera_viewport<'q>(
    camera: &Camera,
    touches: Option<&Res<Touches>>,
    primary_window: &'q Query<(Entity, &Window), With<PrimaryWindow>>,
    other_windows: &'q Query<(Entity, &Window), Without<PrimaryWindow>>,
) -> Option<(Entity, &'q Window)> {
    // First check if cursor is in the same window as this camera
    if let RenderTarget::Window(win_ref) = camera.target {
        let Some((window_entity, window)) = (match win_ref {
            WindowRef::Primary => primary_window.get_single().ok(),
            WindowRef::Entity(entity) => other_windows.get(entity).ok(),
        }) else {
            // Window does not exist - maybe it was closed and the
            // camera not cleaned up
            return None;
        };
        // Is the cursor/touch in this window?
        // Note: there's a bug in winit that causes
        // `window.cursor_position()` to return a `Some` value even if
        // the cursor is not in this window, in very specific cases.
        // See: https://github.com/Plonq/bevy_panorbit_camera/issues/22
        if let Some(input_position) = window.cursor_position().or(touches
            .map_or_else(
                || None,
                |v| {
                    v.iter_just_pressed()
                        .collect::<Vec<_>>()
                        .first()
                        .map(|touch| touch.position())
                },
            ))
        {
            // Now check if cursor is within this camera's viewport
            if let Some(Rect { min, max }) = camera.logical_viewport_rect() {
                // Window coordinates have Y starting at the bottom, so
                // we need to reverse the y component before comparing
                // with the viewport rect
                let cursor_in_vp = input_position.x > min.x
                    && input_position.x < max.x
                    && input_position.y > min.y
                    && input_position.y < max.y;
                if cursor_in_vp {
                    return Some((window_entity, window));
                }
            }
        }
    }
    None
}

/// Get the camera entity that renders to the viewport under the mouse
/// cursor with highest rendering order.
pub fn get_camera_entity_from_cursor_position(
    cameras_query: &Query<(Entity, &Camera)>,
    primary_window: &Query<(Entity, &Window), With<PrimaryWindow>>,
    other_windows: &Query<(Entity, &Window), Without<PrimaryWindow>>,
) -> Option<Entity> {
    let mut camera_entity = None;
    let mut max_cam_order = 0;
    for (entity, camera) in cameras_query.iter() {
        if get_window_if_cursor_in_camera_viewport(
            camera,
            None,
            primary_window,
            other_windows,
        )
        .is_some()
        {
            // Only set if camera order is higher. This may
            // overwrite a previous value in the case the viewport
            // is overlapping another viewport.
            if camera.order >= max_cam_order {
                camera_entity = Some(entity);
                max_cam_order = camera.order;
            }
        }
    }
    camera_entity
}

/// Gather data about the active viewport, i.e. the viewport the user is
/// interacting with.
/// Enables multiple viewports/windows.
#[allow(clippy::too_many_arguments)]
fn active_viewport_data_system(
    mut active_cam: ResMut<ActiveCameraData>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    scroll_events: EventReader<MouseWheel>,
    touches: Res<Touches>,
    primary_window: Query<(Entity, &Window), With<PrimaryWindow>>,
    other_windows: Query<(Entity, &Window), Without<PrimaryWindow>>,
    orbit_fly_cameras: Query<(
        Entity,
        &Camera,
        Option<&OrbitCameraController>,
        Option<&FlyCameraController>,
    )>,
    #[cfg(feature = "bevy_egui")] egui_wants_focus: Res<EguiWantsFocus>,
) {
    let mut new_resource = ActiveCameraData::default();
    let mut max_cam_order = 0;
    let mut has_input = false;
    for (entity, camera, orbit_controller_opt, fly_controller_opt) in
        orbit_fly_cameras.iter()
    {
        if orbit_controller_opt.is_none() && fly_controller_opt.is_none() {
            continue;
        }

        let mut drag_just_activated = false;
        if let Some(orbit_controller) = orbit_controller_opt {
            if orbit_controller.is_enabled {
                drag_just_activated = drag_just_activated
                    || (input::orbit_just_pressed(
                        orbit_controller,
                        &mouse_input,
                        &key_input,
                    ) || input::pan_just_pressed(
                        orbit_controller,
                        &mouse_input,
                        &key_input,
                    ));
            }
        }
        if let Some(fly_controller) = fly_controller_opt {
            if fly_controller.is_enabled {
                drag_just_activated = drag_just_activated
                    || input::rotate_just_pressed(
                        fly_controller,
                        &mouse_input,
                        &key_input,
                    );
            }
        }

        let input_just_activated = drag_just_activated
            || !scroll_events.is_empty()
            || (touches.iter_just_pressed().count() > 0
                && touches.iter_just_pressed().count()
                    == touches.iter().count());
        if input_just_activated {
            has_input = true;

            #[allow(unused_mut, unused_assignments)]
            let mut should_get_input = true;
            #[cfg(feature = "bevy_egui")]
            {
                should_get_input =
                    !egui_wants_focus.prev && !egui_wants_focus.curr;
            }
            if should_get_input {
                if let Some((window_entity, window)) =
                    get_window_if_cursor_in_camera_viewport(
                        camera,
                        Some(&touches),
                        &primary_window,
                        &other_windows,
                    )
                {
                    // Only set if camera order is higher. This may
                    // overwrite a previous value in the case the viewport
                    // is overlapping another viewport.
                    if camera.order >= max_cam_order {
                        new_resource = ActiveCameraData {
                            entity: Some(entity),
                            viewport_size: camera.logical_viewport_size(),
                            window_size: Some(Vec2::new(
                                window.width(),
                                window.height(),
                            )),
                            manual: false,
                            window_entity: Some(window_entity),
                        };
                        max_cam_order = camera.order;
                    }
                }
            }
        }
    }

    if has_input {
        active_cam.set_if_neq(new_resource);
    }
}

/// Grap, wrap around and center cursor when needed
fn wrap_grab_center_cursor_system(
    active_cam: Res<ActiveCameraData>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
    orbit_fly_cameras: Query<(
        &Camera,
        Option<&OrbitCameraController>,
        Option<&FlyCameraController>,
    )>,
    mut cursor_start_pos: Local<Option<Vec2>>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    let Some(window_entity) = active_cam.window_entity else {
        return;
    };
    let Some(winit_window) = winit_windows.get_window(window_entity) else {
        return;
    };
    let Ok(mut window) = windows.get_mut(window_entity) else {
        return;
    };
    let Some(camera_entity) = active_cam.entity else {
        return;
    };
    let Ok((camera, orbit_controller_opt, fly_controller_opt)) =
        orbit_fly_cameras.get(camera_entity)
    else {
        return;
    };
    if orbit_controller_opt.is_none() && fly_controller_opt.is_none() {
        return;
    }

    let mut drag_just_activated = false;
    let mut drag_just_released = false;
    let mut wrap_cursor = false;
    let mut center_cursor = false;
    if let Some(orbit_controller) = orbit_controller_opt {
        if orbit_controller.is_enabled {
            drag_just_activated = drag_just_activated
                || (input::orbit_just_pressed(
                    orbit_controller,
                    &mouse_input,
                    &key_input,
                ) || input::pan_just_pressed(
                    orbit_controller,
                    &mouse_input,
                    &key_input,
                ));
            let drag_pressed = input::orbit_pressed(
                orbit_controller,
                &mouse_input,
                &key_input,
            ) || input::pan_pressed(
                orbit_controller,
                &mouse_input,
                &key_input,
            );
            drag_just_released = drag_just_released
                || (input::orbit_just_released(
                    orbit_controller,
                    &mouse_input,
                    &key_input,
                ) || input::pan_just_released(
                    orbit_controller,
                    &mouse_input,
                    &key_input,
                ));
            wrap_cursor =
                wrap_cursor || (orbit_controller.wrap_cursor && drag_pressed);
        }
    }
    if let Some(fly_controller) = fly_controller_opt {
        if fly_controller.is_enabled {
            drag_just_activated = drag_just_activated
                || input::rotate_just_pressed(
                    fly_controller,
                    &mouse_input,
                    &key_input,
                );
            drag_just_released = drag_just_released
                || input::rotate_just_released(
                    fly_controller,
                    &mouse_input,
                    &key_input,
                );
            let drag_pressed =
                input::rotate_pressed(fly_controller, &mouse_input, &key_input);
            center_cursor =
                center_cursor || (fly_controller.grab_cursor && drag_pressed);
        }
    }

    let viewport_rect = camera.logical_viewport_rect().unwrap();
    if drag_just_activated {
        *cursor_start_pos = window.cursor_position();
        if wrap_cursor {
            // HACK: No need to grab/lock cursor if warp worked with all
            // window manager on all platforms
            // TODO: This grab cursor works differently on X11, Wayland,
            // window, mac, android, ios, ... Test more OS
            // For now it is only tested on X11 and Wayland
            // - On X11 no lock mode works, but we can set the cursor position
            //   manually.
            // - On Wayland only `Locked` works and cannot set cursor position
            //   unless it is locked according to message but can never be
            //   set according to tests.
            window.cursor.grab_mode = CursorGrabMode::Locked;
            // window.cursor.grab_mode = CursorGrabMode::Confined;
        }
        if center_cursor {
            let center = viewport_rect.center();
            // HACK: Avoid Wayland error message
            let _ = winit_window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked);
            // End of hack
            window.cursor.grab_mode = CursorGrabMode::Locked;
            // window.cursor.visible = false;
            // FIXME: Does not work in Wayland
            window.set_cursor_position(Some(center));
        }
    } else if drag_just_released {
        *cursor_start_pos = None;
        window.cursor.grab_mode = CursorGrabMode::None;
        // window.cursor.visible = true;
    }
    // Only wrap/center/grab if dragging started in the viewport.
    if cursor_start_pos.is_some()
        && cursor_start_pos.unwrap().x >= viewport_rect.min.x
        && cursor_start_pos.unwrap().x <= viewport_rect.max.x
        && cursor_start_pos.unwrap().y >= viewport_rect.min.y
        && cursor_start_pos.unwrap().y <= viewport_rect.max.y
    {
        if wrap_cursor {
            if let Some(mut pos) = window.cursor_position() {
                if pos.x <= viewport_rect.min.x {
                    pos.x = viewport_rect.max.x;
                } else if pos.x >= viewport_rect.max.x {
                    pos.x = viewport_rect.min.x;
                }
                if pos.y <= viewport_rect.min.y {
                    pos.y = viewport_rect.max.y;
                } else if pos.y >= viewport_rect.max.y {
                    pos.y = viewport_rect.min.y;
                }
                window.set_cursor_position(Some(pos));
            } else {
                let center = viewport_rect.center();
                window.set_cursor_position(Some(center));
            }
        }
        // Recenter on each frame for platform where lock does not works (X11)
        if center_cursor {
            let center = viewport_rect.center();
            window.set_cursor_position(Some(center));
        }
    }
}

fn switch_to_orbit_camera_controller_system(
    mut ev_read: EventReader<SwitchToOrbitController>,
    mut query: Query<(
        &Transform,
        &mut OrbitCameraController,
        &mut FlyCameraController,
    )>,
) {
    for SwitchToOrbitController { camera_entity } in ev_read.read() {
        if let Ok((transform, mut orbit_controller, mut fly_controller)) =
            query.get_mut(*camera_entity)
        {
            if fly_controller.is_enabled {
                fly_controller.is_enabled = false;
                orbit_controller.is_enabled = true;
                let (yaw, pitch, _roll) =
                    transform.rotation.to_euler(EulerRot::YXZ);
                orbit_controller.yaw = Some(yaw);
                orbit_controller.pitch = Some(-pitch);
                orbit_controller.focus = transform.translation
                    + (transform.forward() * orbit_controller.radius.unwrap());
            }
        } else {
            warn!(
                "Camera not found while trying to swith to OrbitCameraController"
            );
        }
    }
}

fn switch_to_fly_camera_controller_system(
    mut ev_read: EventReader<SwitchToFlyController>,
    mut query: Query<(
        &mut Transform,
        &mut OrbitCameraController,
        &mut FlyCameraController,
        &mut Projection,
        &mut OtherProjection,
    )>,
) {
    for SwitchToFlyController { camera_entity } in ev_read.read() {
        if let Ok((
            mut transform,
            mut orbit_controller,
            mut fly_controller,
            mut projection,
            mut next_projection,
        )) = query.get_mut(*camera_entity)
        {
            if orbit_controller.is_enabled {
                orbit_controller.is_enabled = false;
                fly_controller.is_enabled = true;
                // FIXME: commenting this makes fly mode works with ortho too
                // but zoom and sensitivity behave wierdly
                if let Projection::Orthographic(_) = *projection {
                    switch_camera_projection(
                        &orbit_controller,
                        &mut transform,
                        &mut next_projection.0,
                        &mut projection,
                    );
                }
            }
        } else {
            warn!(
                "Camera not found while trying to swith to FlyCameraController"
            );
        }
    }
}

fn switch_camera_projection(
    orbit_controller: &OrbitCameraController,
    transform: &mut Transform,
    next_projection: &mut Projection,
    projection: &mut Projection,
) {
    // Need to update transform/projection
    utils::update_orbit_transform(
        orbit_controller.yaw.unwrap(),
        orbit_controller.pitch.unwrap(),
        orbit_controller.radius.unwrap(),
        orbit_controller.focus,
        transform,
        next_projection,
    );
    std::mem::swap(next_projection, projection);
}

fn switch_camera_projection_system(
    mut ev_read: EventReader<SwitchProjection>,
    mut query: Query<(
        &mut Transform,
        &OrbitCameraController,
        &mut Projection,
        &mut OtherProjection,
    )>,
) {
    for SwitchProjection { camera_entity } in ev_read.read() {
        trace!("Camera projection switch");
        if let Ok((
            mut transform,
            orbit_controller,
            mut projection,
            mut next_projection,
        )) = query.get_mut(*camera_entity)
        {
            // Do not switch if in fly mode, which only work in perspective
            // for now
            // FIXME: We probably need to swicth even if orbit is not enabled
            // this functionality is not really related to the orbit controller
            // appart from the point in the previous commentary
            if orbit_controller.is_enabled {
                switch_camera_projection(
                    orbit_controller,
                    &mut transform,
                    &mut next_projection.0,
                    &mut projection,
                );
            }
        } else {
            warn!("Camera not found while trying to swith to Projection");
        }
    }
}
