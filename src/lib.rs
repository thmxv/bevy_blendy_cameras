use bevy::{
    input::{keyboard::KeyCode, mouse::MouseWheel, ButtonInput},
    prelude::*,
    render::camera::{CameraUpdateSystem, RenderTarget, ScalingMode},
    transform::TransformSystem,
    window::{CursorGrabMode, PrimaryWindow, WindowRef},
    winit::WinitWindows,
};
use bevy_mod_raycast::DefaultRaycastingPlugin;

#[cfg(feature = "bevy_egui")]
use bevy_egui::EguiSet;

pub use crate::{
    fly::{fly_camera_controller_system, FlyCameraController},
    frame::{frame_system, FrameEvent},
    input::{mouse_key_tracker_system, MouseKeyTracker},
    orbit::{orbit_camera_controller_system, OrbitCameraController},
    viewpoints::{viewpoint_system, ViewpointEvent},
};

#[cfg(feature = "bevy_egui")]
pub use crate::egui::EguiWantsFocus;

pub mod fly;
pub mod frame;
mod input;
pub mod orbit;
mod utils;
pub mod viewpoints;

#[cfg(feature = "bevy_egui")]
mod egui;

#[derive(Default, Event)]
pub struct SwitchProjection;

#[derive(Default, Event)]
pub struct SwitchToOrbitController;

#[derive(Default, Event)]
pub struct SwitchToFlyController;

#[derive(Resource, Default)]
pub struct ProjectionResource(Option<Projection>);

/// System set to allow ordering
#[derive(Debug, Clone, Copy, SystemSet, PartialEq, Eq, Hash)]
pub struct EditorCamSystemSet;

/// System set to only run when GUI has NOT the focus
#[derive(Debug, Clone, Copy, SystemSet, PartialEq, Eq, Hash)]
pub struct GuiFocusSystemSet;

pub struct BlendyCamerasPlugin;

impl Plugin for BlendyCamerasPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<DefaultRaycastingPlugin>() {
            app.add_plugins(DefaultRaycastingPlugin);
        }
        app.init_resource::<ActiveCameraData>()
            .init_resource::<MouseKeyTracker>()
            .init_resource::<ProjectionResource>()
            .add_event::<SwitchProjection>()
            .add_event::<SwitchToOrbitController>()
            .add_event::<SwitchToFlyController>()
            .add_event::<ViewpointEvent>()
            .add_event::<FrameEvent>()
            .add_systems(
                //PostUpdate,
                Update,
                (
                    mouse_key_tracker_system,
                    orbit_camera_controller_system,
                    fly_camera_controller_system,
                )
                    .chain()
                    .in_set(GuiFocusSystemSet)
                    .in_set(EditorCamSystemSet)
                    .before(CameraUpdateSystem)
                    .before(TransformSystem::TransformPropagate),
            )
            .add_systems(
                //PostUpdate,
                Update,
                (
                    active_viewport_data_system.run_if(
                        |active_cam: Res<ActiveCameraData>| !active_cam.manual,
                    ),
                    wrap_grab_center_cursor_system
                        .after(active_viewport_data_system),
                    switch_to_fly_camera_controller_system,
                    switch_to_orbit_camera_controller_system,
                    switch_camera_projection_system,
                    viewpoint_system,
                    frame_system,
                )
                    .in_set(EditorCamSystemSet)
                    .before(GuiFocusSystemSet),
            );
        #[cfg(feature = "bevy_egui")]
        {
            app.init_resource::<EguiWantsFocus>()
                .add_systems(
                    //PostUpdate,
                    Update,
                    egui::check_egui_wants_focus
                        .after(EguiSet::InitContexts)
                        //.before(EditorCamSystemSet)
                        .before(GuiFocusSystemSet),
                )
                .configure_sets(
                    //PostUpdate,
                    Update,
                    GuiFocusSystemSet.run_if(resource_equals(EguiWantsFocus {
                        prev: false,
                        curr: false,
                    })),
                );
        }
    }
}

/// Tracks which `PanOrbitCamera` is active (should handle input events),
/// along with the window and viewport dimensions, which are used for scaling
/// mouse motion.
/// `PanOrbitCameraPlugin` manages this resource automatically, in order to
/// support multiple viewports/windows. However, if this doesn't work for you,
/// you can take over and manage it yourself, e.g. when you want to control a
/// camera that is rendering to a texture.
#[derive(Resource, Default, Debug, PartialEq)]
pub struct ActiveCameraData {
    /// ID of the entity with `OrbitCameraController` or `FlyCameraController`
    /// that will handle user input. In other words, this is the camera that
    /// will move when you orbit/pan/zoom.
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
    /// Indicates to `PanOrbitCameraPlugin` that it should not update/overwrite
    /// this resource. If you are manually updating this resource you should
    /// set this to `true`. Note that setting this to `true` will effectively
    /// break multiple viewport/window support unless you manually reimplement
    /// it.
    pub manual: bool,
    // TODO: Doc
    pub window_entity: Option<Entity>,
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
    mut primary_windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut other_windows: Query<(Entity, &mut Window), Without<PrimaryWindow>>,
    orbit_fly_cameras: Query<(
        Entity,
        &Camera,
        Option<&OrbitCameraController>,
        Option<&FlyCameraController>,
    )>,
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
            // First check if cursor is in the same window as this camera
            if let RenderTarget::Window(win_ref) = camera.target {
                let Some((window_entity, window)) = (match win_ref {
                    WindowRef::Primary => primary_windows
                        .get_single_mut()
                        .ok()
                        .map(|v| (v.0, v.1.into_inner())),
                    WindowRef::Entity(entity) => other_windows
                        .get_mut(entity)
                        .ok()
                        .map(|v| (v.0, v.1.into_inner())),
                }) else {
                    // Window does not exist - maybe it was closed and the
                    // camera not cleaned up
                    continue;
                };
                // Is the cursor/touch in this window?
                // Note: there's a bug in winit that causes
                // `window.cursor_position()` to return a `Some` value even if
                // the cursor is not in this window, in very specific cases.
                // See: https://github.com/Plonq/bevy_panorbit_camera/issues/22
                if let Some(input_position) =
                    window.cursor_position().or(touches
                        .iter_just_pressed()
                        .collect::<Vec<_>>()
                        .first()
                        .map(|touch| touch.position()))
                {
                    // Now check if cursor is within this camera's viewport
                    if let Some(Rect { min, max }) =
                        camera.logical_viewport_rect()
                    {
                        // Window coordinates have Y starting at the bottom, so
                        // we need to reverse the y component before comparing
                        // with the viewport rect
                        let cursor_in_vp = input_position.x > min.x
                            && input_position.x < max.x
                            && input_position.y > min.y
                            && input_position.y < max.y;

                        // Only set if camera order is higher. This may
                        // overwrite a previous value in the case the viewport
                        // is overlapping another viewport.
                        if cursor_in_vp && camera.order >= max_cam_order {
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
    }

    if has_input {
        active_cam.set_if_neq(new_resource);
    }
}

pub fn wrap_grab_center_cursor_system(
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
            // TODO: This grab cursor works differently on X11, Wayland,
            // window, mac, android, ios, ... Test more OS
            // For not it is only tested on X11 and Wayland
            // - On X11 no lock works, but we can set the cursor position
            //   manually.
            // - On Wayland only Locked works and cannot set cursor position
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

pub fn switch_to_orbit_camera_controller_system(
    mut ev_read: EventReader<SwitchToOrbitController>,
    mut query: Query<(
        &Transform,
        &mut OrbitCameraController,
        &mut FlyCameraController,
    )>,
) {
    for _ev in ev_read.read() {
        let Ok((transform, mut orbit_controller, mut fly_controller)) =
            query.get_single_mut()
        else {
            return;
        };
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
    }
}

pub fn switch_to_fly_camera_controller_system(
    mut next_projection: ResMut<ProjectionResource>,
    mut ev_read: EventReader<SwitchToFlyController>,
    mut query: Query<(
        &mut Transform,
        &mut OrbitCameraController,
        &mut FlyCameraController,
        &mut Projection,
    )>,
) {
    for _ev in ev_read.read() {
        let Ok((
            mut transform,
            mut orbit_controller,
            mut fly_controller,
            mut projection,
        )) = query.get_single_mut()
        else {
            return;
        };
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
    }
}

fn switch_camera_projection(
    orbit_controller: &OrbitCameraController,
    transform: &mut Transform,
    next_projection: &mut Option<Projection>,
    projection: &mut Projection,
) {
    if next_projection.is_none() {
        *next_projection = match projection {
            Projection::Perspective(_) => {
                Some(Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::FixedVertical(1.0),
                    ..default()
                }))
            }
            Projection::Orthographic(_) => {
                Some(Projection::Perspective(PerspectiveProjection {
                    ..default()
                }))
            }
        }
    }
    if let Some(next) = next_projection {
        // Need to update transform/projection
        utils::update_orbit_transform(
            orbit_controller.yaw.unwrap(),
            orbit_controller.pitch.unwrap(),
            orbit_controller.radius.unwrap(),
            orbit_controller.focus,
            transform,
            next,
        );
        std::mem::swap(next, projection);
    }
}

pub fn switch_camera_projection_system(
    mut next_projection: ResMut<ProjectionResource>,
    mut ev_read: EventReader<SwitchProjection>,
    mut query: Query<(
        &mut Transform,
        &mut OrbitCameraController,
        &mut Projection,
    )>,
) {
    for _ev in ev_read.read() {
        trace!("Camera projection switch");
        // Do not switch if in fly mode, which only work in perspective for now
        let Ok((mut transform, orbit_controller, mut projection)) =
            query.get_single_mut()
        else {
            return;
        };
        if orbit_controller.is_enabled {
            switch_camera_projection(
                &orbit_controller,
                &mut transform,
                &mut next_projection.0,
                &mut projection,
            );
        }
    }
}
