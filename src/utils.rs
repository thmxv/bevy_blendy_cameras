use bevy::prelude::*;

pub fn calculate_from_translation_and_focus(
    translation: Vec3,
    focus: Vec3,
) -> (f32, f32, f32) {
    let comp_vec = translation - focus;
    // let mut radius = comp_vec.length();
    // if radius == 0.0 {
    //     radius = 0.05; // Radius 0 causes problems
    // }
    let radius = comp_vec.length().max(0.05);
    let yaw = if comp_vec.x == 0.0 && comp_vec.z >= 0.0 {
        0.0
    } else {
        (comp_vec.z / (comp_vec.x.powi(2) + comp_vec.z.powi(2)).sqrt()).acos()
    };
    let pitch = (comp_vec.y / radius).asin();
    (yaw, pitch, radius)
}

/// Update `transform` based on yaw, pitch, and the camera's focus and radius
pub fn update_orbit_transform(
    yaw: f32,
    pitch: f32,
    mut radius: f32,
    focus: Vec3,
    transform: &mut Transform,
    projection: &mut Projection,
) {
    if let Projection::Orthographic(ref mut p) = *projection {
        p.scale = radius;
        // (near + far) / 2.0 ensures that objects near `focus` are not clipped
        radius = (p.near + p.far) / 2.0;
    }
    *transform = camera_transform_form_orbit(yaw, pitch, radius, focus);
}

pub fn camera_transform_form_orbit(
    yaw: f32,
    pitch: f32,
    radius: f32,
    focus: Vec3,
) -> Transform {
    let mut transform = Transform::IDENTITY;
    transform.rotation =
        Quat::from_rotation_y(yaw) * Quat::from_rotation_x(-pitch);
    transform.translation = focus + transform.back() * radius;
    transform
}

const EPSILON: f32 = 0.001;
pub fn approx_equal(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
    // (a - b).abs() < 1000.0 * f32::EPSILON
}
