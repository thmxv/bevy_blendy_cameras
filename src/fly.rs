use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{input::MouseKeyTracker, ActiveCameraData};

/// Component to tag an entiy as able to be controlled in "fly mode"
/// The entity must have `Transform` and `Projection` components. Typically
/// you would add `Camera3dBundle` to this entity.
#[derive(Component)]
pub struct FlyCameraController {
    /// Speed with wich the entity is moved. Updated when scrolling mouse wheel
    pub speed: f32,
    /// Key used to move the camera forward
    pub key_move_forward: KeyCode,
    /// Key used to move the camera backward
    pub key_move_backward: KeyCode,
    /// Key used to move the camera left
    pub key_move_left: KeyCode,
    /// Key used to move the camera right
    pub key_move_right: KeyCode,
    /// Key used to move the camera up
    pub key_move_up: KeyCode,
    /// Key used to move the camera down
    pub key_move_down: KeyCode,
    /// Mouse button used to rotate the camera
    pub button_rotate: MouseButton,
    /// Key that must be pressed for the `button_rotate` to work
    pub modifier_rotate: Option<KeyCode>,
    /// Sensitivity of the speed change
    pub speed_sensitivity: f32,
    /// Sensitivity of the movement
    pub move_sensitivity: f32,
    /// Sensitivity of the rotation
    pub rotate_sensitivity: f32,
    /// Do not control the camera if `false`
    pub is_enabled: bool,
    /// Grab the mouse cursor while rotating if `true`
    pub grab_cursor: bool,
}

impl Default for FlyCameraController {
    fn default() -> Self {
        Self {
            speed: 1.0,
            key_move_forward: KeyCode::KeyE,
            key_move_backward: KeyCode::KeyD,
            key_move_left: KeyCode::KeyS,
            key_move_right: KeyCode::KeyF,
            key_move_up: KeyCode::KeyR,
            key_move_down: KeyCode::KeyW,
            button_rotate: MouseButton::Middle,
            modifier_rotate: None,
            speed_sensitivity: 1.0,
            move_sensitivity: 1.0,
            rotate_sensitivity: 1.0,
            is_enabled: true,
            grab_cursor: true,
        }
    }
}

pub(crate) fn fly_camera_controller_system(
    active_cam: Res<ActiveCameraData>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_key_tracker: Res<MouseKeyTracker>,
    time: Res<Time>,
    mut fly_cameras: Query<(Entity, &mut FlyCameraController, &mut Transform)>,
) {
    for (entity, mut controller, mut transform) in fly_cameras.iter_mut() {
        if controller.is_enabled && active_cam.entity == Some(entity) {
            // TODO: remove duplicated code with orbit?
            let rotate =
                mouse_key_tracker.rotate * controller.rotate_sensitivity;
            let scroll_line =
                mouse_key_tracker.scroll_line * controller.speed_sensitivity;
            let scroll_pixel =
                mouse_key_tracker.scroll_pixel * controller.speed_sensitivity;

            if (scroll_line + scroll_pixel).abs() > 0.0 {
                let old_speed = controller.speed;
                let line_delta = scroll_line * old_speed * 0.1;
                let pixel_delta = scroll_pixel * old_speed * 0.1;
                let speed_delta = line_delta + pixel_delta;
                controller.speed += speed_delta;
                // NOTE: Avoid speed going down to 0.0 or to high but maybe
                // 0.05/100.0 mps are not right. If move sensitivity is 1.0,
                // those values correspond to 0.18/360 kmph
                controller.speed = controller.speed.clamp(0.05, 100.0);
            }
            if rotate.length_squared() > 0.0 {
                // Use window size for rotation otherwise the sensitivity
                // is far too high for small viewports
                // TODO: remove duplicated code with orbit
                if let Some(win_size) = active_cam.window_size {
                    let delta_yaw = rotate.x / win_size.x * PI * 2.0;
                    let delta_pitch = rotate.y / win_size.y * PI;
                    // Order is important to avoid unwanted roll
                    let (mut yaw, mut pitch, _) =
                        transform.rotation.to_euler(EulerRot::YXZ);
                    yaw -= delta_yaw;
                    pitch -= delta_pitch;
                    transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw)
                        * Quat::from_axis_angle(Vec3::X, pitch);
                }
            }
            let forward = Vec3::from(transform.forward());
            let left = Vec3::from(transform.left());
            let up = Vec3::from(transform.up());
            let mut translation = Vec3::ZERO;
            for key in key_input.get_pressed() {
                if *key == controller.key_move_forward {
                    translation += forward;
                }
                if *key == controller.key_move_backward {
                    translation -= forward;
                }
                if *key == controller.key_move_left {
                    translation += left;
                }
                if *key == controller.key_move_right {
                    translation -= left;
                }
                if *key == controller.key_move_up {
                    translation += up;
                }
                if *key == controller.key_move_down {
                    translation -= up;
                }
            }
            translation = translation.normalize_or_zero();
            translation *= controller.speed
                * controller.move_sensitivity
                * time.delta_seconds();
            transform.translation += translation;
        }
    }
}
