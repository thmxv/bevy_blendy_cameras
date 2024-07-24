use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{input::MouseKeyTracker, ActiveCameraData};

#[derive(Component)]
pub struct FlyCameraController {
    pub speed: f32,
    pub key_move_forward: KeyCode,
    pub key_move_backward: KeyCode,
    pub key_move_left: KeyCode,
    pub key_move_right: KeyCode,
    pub key_move_up: KeyCode,
    pub key_move_down: KeyCode,
    pub button_rotate: MouseButton,
    pub modifier_rotate: Option<KeyCode>,
    pub speed_sensitivity: f32,
    pub move_sensitivity: f32,
    pub rotate_sensitivity: f32,
    pub is_enabled: bool,
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

pub fn fly_camera_controller_system(
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
