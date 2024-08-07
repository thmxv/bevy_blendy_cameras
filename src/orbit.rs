use std::f32::consts::PI;

use bevy::{
    ecs::component::StorageType, prelude::*, render::camera::ScalingMode,
};
use bevy_mod_raycast::prelude::*;

use crate::{
    input::{self, MouseKeyTracker},
    raycast::BlendyCamerasRaycastSet,
    utils, ActiveCameraData, OtherProjection,
};

/// Component to tag an entiy as able to be controlled by orbiting, panning
/// and zooming.
/// The entity must have `Transform` and `Projection` components. Typically
/// you would add `Camera3dBundle` to this entity.
pub struct OrbitCameraController {
    /// The point the camera looks at. The camera also orbit around and zoom
    /// to that point if `auto_depth` and `zoom_to_mouse_position` are not set.
    /// This is updated when panning or when zooming to the mouse position
    /// or when zooming or orbiting when `auto_depth` is set.
    pub focus: Vec3,
    /// The distance between the camera and the `focus`.
    /// If set to `None`, it will be calculated from the camera's current
    /// position during intialization.
    /// Automatically updated
    pub radius: Option<f32>,
    /// Rotation in radian around the global Y axis.
    /// If set to `None`, it will be calculated from the camera's current
    /// position during intialization.
    /// Automatically updated
    pub yaw: Option<f32>,
    /// Rotation in radian around a global horizontal axis perpendicular to
    /// the view direction.
    /// If set to `None`, it will be calculated from the camera's current
    /// position during intialization.
    /// Automatically updated
    pub pitch: Option<f32>,
    /// Sentitivity of the orbiting motion
    pub orbit_sensitivity: f32,
    /// Sentitivity of the panning motion
    pub pan_sensitivity: f32,
    /// Sentitivity of the zooming motion
    pub zoom_sensitivity: f32,
    /// Mouse button used to orbit the camera
    pub button_orbit: MouseButton,
    /// Key that must be pressed for the `button_orbit` to work
    pub modifier_orbit: Option<KeyCode>,
    /// Mouse button used to pan the camera
    pub button_pan: MouseButton,
    /// Key that must be pressed for the `button_pan` to work
    pub modifier_pan: Option<KeyCode>,
    /// Do not control the camera if `false`
    pub is_enabled: bool,
    /// Whether [`OrbitCameraController`] has been initialized
    pub is_initialized: bool,
    /// Enable zooming in the direction of the mouse cursor
    pub zoom_to_mouse_position: bool,
    /// Enable setting the focus to the distance of the geometry under the
    /// mouse cursor while moving the camera. This will cause the camera to
    /// orbit around the geometry under the mouse cursor and zoom speed beeing
    /// relative to the distance to this geometry point.
    pub auto_depth: bool,
    /// Wrap the mouse cursor while rotating or panning if `true`.
    /// Because wrapping is not working on all platfrom or with all windowing
    /// system, this will also cause a mouse grab/lock.
    pub wrap_cursor: bool,
    /// Whether the camera is currently upside down. Inverting the direction
    /// of rotation to be more intuitive.
    /// Automatically updated
    pub is_upside_down: bool,
    /// Whether to update the camera's transform regardless of whether there
    /// are any changes/input.
    /// Set this to `true` if you want to modify values directly.
    /// This will be automatically set back to `false` after one frame.
    pub force_update: bool,
}

impl Component for OrbitCameraController {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks
            .on_add(|mut world, entity, _component_id| {
                let projection = world.get::<Projection>(entity).unwrap();
                let projection = match projection {
                    Projection::Perspective(_) => {
                        Projection::Orthographic(OrthographicProjection {
                            scaling_mode: ScalingMode::FixedVertical(1.0),
                            ..default()
                        })
                    }
                    Projection::Orthographic(_) => {
                        Projection::Perspective(PerspectiveProjection {
                            ..default()
                        })
                    }
                };
                world
                    .commands()
                    .entity(entity)
                    .insert(OtherProjection(projection))
                    // TODO: Only insert if camera is active
                    .insert(
                        RaycastSource::<BlendyCamerasRaycastSet>::new_cursor(),
                    );
            })
            .on_remove(|mut world, entity, _component_id| {
                world
                    .commands()
                    .entity(entity)
                    .remove::<RaycastSource<BlendyCamerasRaycastSet>>()
                    .remove::<OtherProjection>();
            });
    }
}

impl Default for OrbitCameraController {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: None,
            yaw: None,
            pitch: None,
            orbit_sensitivity: 1.0,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 1.0,
            button_orbit: MouseButton::Middle,
            modifier_orbit: None,
            button_pan: MouseButton::Middle,
            modifier_pan: Some(KeyCode::ShiftLeft),
            is_enabled: true,
            is_initialized: false,
            zoom_to_mouse_position: true,
            auto_depth: true,
            wrap_cursor: true,
            is_upside_down: false,
            force_update: false,
        }
    }
}

impl OrbitCameraController {
    pub(crate) fn initialize_if_necessary(
        &mut self,
        transform: &mut Transform,
        projection: &mut Projection,
    ) {
        if !self.is_initialized {
            let (yaw, pitch, radius) =
                utils::calculate_from_translation_and_focus(
                    transform.translation,
                    self.focus,
                );
            let &mut yaw = self.yaw.get_or_insert(yaw);
            let &mut pitch = self.pitch.get_or_insert(pitch);
            let &mut radius = self.radius.get_or_insert(radius);
            utils::update_orbit_transform(
                yaw, pitch, radius, self.focus, transform, projection,
            );
            self.is_initialized = true;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn orbit_camera(
    controller: &mut Mut<OrbitCameraController>,
    raycast_source: &RaycastSource<BlendyCamerasRaycastSet>,
    transform: &Mut<Transform>,
    projection: &Mut<Projection>,
    active_cam: &Res<ActiveCameraData>,
    key_input: &Res<ButtonInput<KeyCode>>,
    mouse_input: &Res<ButtonInput<MouseButton>>,
    mouse_key_tracker: &Res<MouseKeyTracker>,
    pivot_point: &mut Local<Vec3>,
) -> bool {
    // Update pivot point when needed
    if (controller.auto_depth || controller.zoom_to_mouse_position)
        && (input::orbit_just_pressed(controller, mouse_input, key_input)
            || input::pan_just_pressed(controller, mouse_input, key_input)
            || mouse_key_tracker.scroll_line != 0.0
            || mouse_key_tracker.scroll_pixel != 0.0)
    {
        if let Some(cursor_ray) = raycast_source.get_ray() {
            if let Some((_entity, hit)) =
                raycast_source.get_nearest_intersection()
            {
                **pivot_point = hit.position();
                if controller.auto_depth {
                    let camera_transform = match **projection {
                        Projection::Perspective(_) => **transform,
                        Projection::Orthographic(_) => {
                            utils::camera_transform_form_orbit(
                                controller.yaw.unwrap(),
                                controller.pitch.unwrap(),
                                controller.radius.unwrap(),
                                controller.focus,
                            )
                        }
                    };
                    let camera_to_pivot =
                        **pivot_point - camera_transform.translation;
                    let pivot_distance = camera_to_pivot.length();
                    let factor = camera_transform
                        .forward()
                        .dot(camera_to_pivot.normalize());
                    let new_radius = pivot_distance * factor;
                    let new_radius = new_radius.max(0.05);
                    let new_focus = camera_transform.translation
                        + (camera_transform.forward() * new_radius);
                    if let Projection::Perspective(_) = **projection {
                        controller.radius = Some(new_radius);
                    }
                    controller.focus = new_focus;
                }
            } else {
                **pivot_point = match **projection {
                    // NOTE: cursor_ray.origin is not the camera
                    // position it is probably on the near plane
                    Projection::Perspective(_) => {
                        let factor = transform
                            .forward()
                            .dot(cursor_ray.direction.into());
                        transform.translation
                            + cursor_ray.direction
                                * (controller.radius.unwrap() / factor)
                    }
                    Projection::Orthographic(ref p) => {
                        let radius_minus_near = (p.far - p.near) / 2.0;
                        cursor_ray.origin
                            + cursor_ray.direction * radius_minus_near
                    }
                };
            }
        }
    }
    let orbit = mouse_key_tracker.orbit * controller.orbit_sensitivity;
    let mut pan = mouse_key_tracker.pan * controller.pan_sensitivity;
    let scroll_line =
        mouse_key_tracker.scroll_line * controller.zoom_sensitivity;
    let scroll_pixel =
        mouse_key_tracker.scroll_pixel * controller.zoom_sensitivity;
    let orbit_button_changed = mouse_key_tracker.orbit_button_changed;

    if orbit_button_changed {
        let up = transform.rotation * Vec3::Y;
        controller.is_upside_down = up.y <= 0.0;
    }
    let mut has_moved = false;
    // TODO: Draw a sceen space 2D disk for rotation center
    if orbit.length_squared() > 0.0 {
        // Use window size for rotation otherwise the sensitivity
        // is far too high for small viewports
        if let Some(win_size) = active_cam.window_size {
            let delta_yaw = {
                let delta = orbit.x / win_size.x * PI * 2.0;
                if controller.is_upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_pitch = orbit.y / win_size.y * PI;
            let pre_yaw = controller.yaw.unwrap();
            let pre_pitch = controller.pitch.unwrap();
            controller.yaw = controller.yaw.map(|value| value - delta_yaw);
            controller.pitch =
                controller.pitch.map(|value| value + delta_pitch);
            if controller.auto_depth {
                let mut transform_tmp = utils::camera_transform_form_orbit(
                    pre_yaw,
                    pre_pitch,
                    controller.radius.unwrap(),
                    controller.focus,
                );
                let yaw = Quat::from_rotation_y(-delta_yaw);
                let pitch = Quat::from_rotation_x(-delta_pitch);
                let pitch_global = transform_tmp.rotation
                    * pitch
                    * transform_tmp.rotation.inverse();
                transform_tmp.rotate_around(**pivot_point, yaw * pitch_global);
                controller.focus = transform_tmp.translation
                    + (transform_tmp.forward() * controller.radius.unwrap());
            }
            has_moved = true;
        }
    }
    if pan.length_squared() > 0.0 {
        // Make panning distance independent of resolution and FOV,
        if let Some(vp_size) = active_cam.viewport_size {
            let mut multiplier = 1.0;
            match **projection {
                Projection::Perspective(ref p) => {
                    pan *= Vec2::new(p.fov * p.aspect_ratio, p.fov) / vp_size;
                    // Make panning proportional to distance away from
                    // focus point
                    if let Some(radius) = controller.radius {
                        multiplier = radius;
                    }
                }
                Projection::Orthographic(ref p) => {
                    pan *= Vec2::new(p.area.width(), p.area.height()) / vp_size;
                }
            }
            // Translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            let translation = (right + up) * multiplier;
            controller.focus += translation;
            has_moved = true;
        }
    }
    if (scroll_line + scroll_pixel).abs() > 0.0 {
        let old_radius = controller.radius.unwrap();
        // Calculate the impact of scrolling on the reference value
        let line_delta = -scroll_line * old_radius * 0.2;
        let pixel_delta = -scroll_pixel * old_radius * 0.2;
        let radius_delta = line_delta + pixel_delta;
        // Update the target value
        controller.radius = controller.radius.map(|value| value + radius_delta);
        // If it is pixel-based scrolling, add it directly to the
        // current value
        // controller.radius =
        //     controller.radius.map(|value| value + pixel_delta);
        if controller.zoom_to_mouse_position {
            // TODO: clean
            match **projection {
                Projection::Perspective(_) => {
                    let old_camera_pos = transform.translation;
                    let old_camera_to_pivot = **pivot_point - old_camera_pos;
                    let mouse_direction = old_camera_to_pivot.normalize();
                    let factor = transform.forward().dot(mouse_direction);
                    let new_camera_pos = old_camera_pos
                        + mouse_direction * (-radius_delta / factor);
                    let new_focus = new_camera_pos
                        + transform.forward() * controller.radius.unwrap();
                    controller.focus = new_focus;
                }
                Projection::Orthographic(_) => {
                    let focus_to_pivot = **pivot_point - controller.focus;
                    let focus_to_pivot =
                        transform.rotation.inverse() * focus_to_pivot;
                    let focus_to_pivot = focus_to_pivot.xy();
                    let focus_to_pivot = Vec3::from((focus_to_pivot, 0.0));
                    let focus_to_pivot = transform.rotation * focus_to_pivot;
                    let new_radius = controller.radius.unwrap();
                    controller.focus +=
                        focus_to_pivot * (1.0 - (new_radius / old_radius));
                }
            }
        }
        has_moved = true;
    }
    has_moved
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn orbit_camera_controller_system(
    active_cam: Res<ActiveCameraData>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mouse_key_tracker: Res<MouseKeyTracker>,
    mut orbit_cameras: Query<(
        Entity,
        &mut OrbitCameraController,
        &RaycastSource<BlendyCamerasRaycastSet>,
        &mut Transform,
        &mut Projection,
    )>,
    mut pivot_point: Local<Vec3>,
    //mut gizmos: Gizmos,
) {
    for (
        entity,
        mut controller,
        raycast_source,
        mut transform,
        mut projection,
    ) in orbit_cameras.iter_mut()
    {
        controller.initialize_if_necessary(&mut transform, &mut projection);
        let mut has_moved = false;
        if controller.is_enabled && active_cam.entity == Some(entity) {
            has_moved = orbit_camera(
                &mut controller,
                raycast_source,
                &transform,
                &projection,
                &active_cam,
                &key_input,
                &mouse_input,
                &mouse_key_tracker,
                &mut pivot_point,
            );
            //gizmos.sphere(
            //    controller.focus,
            //    Quat::IDENTITY,
            //    0.2,
            //    bevy::color::palettes::css::AQUAMARINE,
            //);
            //gizmos.sphere(
            //    *pivot_point,
            //    Quat::IDENTITY,
            //    0.2,
            //    bevy::color::palettes::css::ORANGE_RED,
            //);
        }
        // Update the camera's transform based on current values
        if let (Some(yaw), Some(pitch), Some(radius)) =
            (controller.yaw, controller.pitch, controller.radius)
        {
            if has_moved || controller.force_update {
                utils::update_orbit_transform(
                    yaw,
                    pitch,
                    radius,
                    controller.focus,
                    &mut transform,
                    &mut projection,
                );
                controller.force_update = false;
            }
        }
    }
}
