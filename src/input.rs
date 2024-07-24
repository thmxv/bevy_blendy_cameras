use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::{
    fly::FlyCameraController, orbit::OrbitCameraController, ActiveCameraData,
};

#[derive(Resource, Default, Debug)]
pub struct MouseKeyTracker {
    pub orbit: Vec2,
    pub pan: Vec2,
    pub scroll_line: f32,
    pub scroll_pixel: f32,
    pub orbit_button_changed: bool,
    pub rotate: Vec2,
}

// TODO: Maybe make 2 systems
pub fn mouse_key_tracker_system(
    mut camera_movement: ResMut<MouseKeyTracker>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    active_cam: Res<ActiveCameraData>,
    orbit_cameras: Query<&OrbitCameraController>,
    fly_cameras: Query<&FlyCameraController>,
) {
    if let Some(active_entity) = active_cam.entity {
        // TODO: clean, remove duplicate code
        if let Ok(orbit_controller) = orbit_cameras.get(active_entity) {
            if orbit_controller.is_enabled {
                let mut orbit = Vec2::ZERO;
                let mut pan = Vec2::ZERO;
                let mut scroll_line = 0.0;
                let mut scroll_pixel = 0.0;
                let mut orbit_button_changed = false;
                let rotate = Vec2::ZERO;

                // Collect input deltas
                let mouse_delta =
                    mouse_motion.read().map(|event| event.delta).sum::<Vec2>();
                let (scroll_line_delta, scroll_pixel_delta) = scroll_events
                    .read()
                    .map(|event| match event.unit {
                        MouseScrollUnit::Line => (event.y, 0.0),
                        MouseScrollUnit::Pixel => (0.0, event.y * 0.005),
                    })
                    .fold((0.0, 0.0), |acc, item| {
                        (acc.0 + item.0, acc.1 + item.1)
                    });

                // Orbit and pan
                if orbit_pressed(orbit_controller, &mouse_input, &key_input) {
                    orbit += mouse_delta;
                } else if pan_pressed(
                    orbit_controller,
                    &mouse_input,
                    &key_input,
                ) {
                    // Pan only if we're not rotating at the moment
                    pan += mouse_delta;
                }

                // Zoom
                scroll_line += scroll_line_delta;
                scroll_pixel += scroll_pixel_delta;

                // Other
                if orbit_just_pressed(
                    orbit_controller,
                    &mouse_input,
                    &key_input,
                ) || orbit_just_released(
                    orbit_controller,
                    &mouse_input,
                    &key_input,
                ) {
                    orbit_button_changed = true;
                }

                camera_movement.orbit = orbit;
                camera_movement.pan = pan;
                camera_movement.scroll_line = scroll_line;
                camera_movement.scroll_pixel = scroll_pixel;
                camera_movement.orbit_button_changed = orbit_button_changed;
                camera_movement.rotate = rotate;
            }
        }
        if let Ok(fly_controller) = fly_cameras.get(active_entity) {
            if fly_controller.is_enabled {
                let orbit = Vec2::ZERO;
                let pan = Vec2::ZERO;
                let mut scroll_line = 0.0;
                let mut scroll_pixel = 0.0;
                let orbit_button_changed = false;
                let mut rotate = Vec2::ZERO;

                // Collect input deltas
                let mouse_delta =
                    mouse_motion.read().map(|event| event.delta).sum::<Vec2>();
                let (scroll_line_delta, scroll_pixel_delta) = scroll_events
                    .read()
                    .map(|event| match event.unit {
                        MouseScrollUnit::Line => (event.y, 0.0),
                        MouseScrollUnit::Pixel => (0.0, event.y * 0.005),
                    })
                    .fold((0.0, 0.0), |acc, item| {
                        (acc.0 + item.0, acc.1 + item.1)
                    });

                // Rotate
                if rotate_pressed(fly_controller, &mouse_input, &key_input) {
                    rotate += mouse_delta;
                }

                // Speed
                scroll_line += scroll_line_delta;
                scroll_pixel += scroll_pixel_delta;

                camera_movement.orbit = orbit;
                camera_movement.pan = pan;
                camera_movement.scroll_line = scroll_line;
                camera_movement.scroll_pixel = scroll_pixel;
                camera_movement.orbit_button_changed = orbit_button_changed;
                camera_movement.rotate = rotate;
            }
        }
    }
}

pub fn orbit_pressed(
    pan_orbit: &OrbitCameraController,
    mouse_input: &Res<ButtonInput<MouseButton>>,
    key_input: &Res<ButtonInput<KeyCode>>,
) -> bool {
    let is_pressed = pan_orbit
        .modifier_orbit
        .map_or(true, |modifier| key_input.pressed(modifier))
        && mouse_input.pressed(pan_orbit.button_orbit);

    is_pressed
        && pan_orbit
            .modifier_pan
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

pub fn orbit_just_pressed(
    pan_orbit: &OrbitCameraController,
    mouse_input: &Res<ButtonInput<MouseButton>>,
    key_input: &Res<ButtonInput<KeyCode>>,
) -> bool {
    let just_pressed = pan_orbit
        .modifier_orbit
        .map_or(true, |modifier| key_input.pressed(modifier))
        && (mouse_input.just_pressed(pan_orbit.button_orbit));

    just_pressed
        && pan_orbit
            .modifier_pan
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

pub fn orbit_just_released(
    pan_orbit: &OrbitCameraController,
    mouse_input: &Res<ButtonInput<MouseButton>>,
    key_input: &Res<ButtonInput<KeyCode>>,
) -> bool {
    let just_released = pan_orbit
        .modifier_orbit
        .map_or(true, |modifier| key_input.pressed(modifier))
        && (mouse_input.just_released(pan_orbit.button_orbit));

    just_released
        && pan_orbit
            .modifier_pan
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

pub fn pan_pressed(
    pan_orbit: &OrbitCameraController,
    mouse_input: &Res<ButtonInput<MouseButton>>,
    key_input: &Res<ButtonInput<KeyCode>>,
) -> bool {
    let is_pressed = pan_orbit
        .modifier_pan
        .map_or(true, |modifier| key_input.pressed(modifier))
        && mouse_input.pressed(pan_orbit.button_pan);

    is_pressed
        && pan_orbit
            .modifier_orbit
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

pub fn pan_just_pressed(
    pan_orbit: &OrbitCameraController,
    mouse_input: &Res<ButtonInput<MouseButton>>,
    key_input: &Res<ButtonInput<KeyCode>>,
) -> bool {
    let just_pressed = pan_orbit
        .modifier_pan
        .map_or(true, |modifier| key_input.pressed(modifier))
        && (mouse_input.just_pressed(pan_orbit.button_pan));

    just_pressed
        && pan_orbit
            .modifier_orbit
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

pub fn pan_just_released(
    pan_orbit: &OrbitCameraController,
    mouse_input: &Res<ButtonInput<MouseButton>>,
    key_input: &Res<ButtonInput<KeyCode>>,
) -> bool {
    let just_released = pan_orbit
        .modifier_pan
        .map_or(true, |modifier| key_input.pressed(modifier))
        && (mouse_input.just_released(pan_orbit.button_pan));

    just_released
        && pan_orbit
            .modifier_orbit
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

pub fn rotate_pressed(
    fly_controller: &FlyCameraController,
    mouse_input: &Res<ButtonInput<MouseButton>>,
    key_input: &Res<ButtonInput<KeyCode>>,
) -> bool {
    fly_controller
        .modifier_rotate
        .map_or(true, |modifier| key_input.pressed(modifier))
        && mouse_input.pressed(fly_controller.button_rotate)
}

pub fn rotate_just_pressed(
    fly_controller: &FlyCameraController,
    mouse_input: &Res<ButtonInput<MouseButton>>,
    key_input: &Res<ButtonInput<KeyCode>>,
) -> bool {
    fly_controller
        .modifier_rotate
        .map_or(true, |modifier| key_input.pressed(modifier))
        && (mouse_input.just_pressed(fly_controller.button_rotate))
}

pub fn rotate_just_released(
    fly_orbit: &FlyCameraController,
    mouse_input: &Res<ButtonInput<MouseButton>>,
    key_input: &Res<ButtonInput<KeyCode>>,
) -> bool {
    fly_orbit
        .modifier_rotate
        .map_or(true, |modifier| key_input.pressed(modifier))
        && (mouse_input.just_released(fly_orbit.button_rotate))
}
