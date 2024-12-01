use bevy::{picking::mesh_picking::ray_cast::RayMeshHit, prelude::*};

pub(crate) fn get_cursor_ray(
    camera: &Camera,
    global_transform: &GlobalTransform,
    window: &Window,
) -> Option<Ray3d> {
    window.cursor_position().map(|pos| (window, pos)).and_then(
        |(window, cursor_pos)| {
            let mut viewport_cursor = cursor_pos;
            if let Some(viewport) = &camera.viewport {
                viewport_cursor -= viewport.physical_position.as_vec2()
                    / window.scale_factor();
            }
            camera
                .viewport_to_world(global_transform, viewport_cursor)
                .ok()
        },
    )
}

pub(crate) fn get_nearest_intersection<'a>(
    ray_cast: &'a mut MeshRayCast,
    ray: Ray3d,
) -> Option<&'a (Entity, RayMeshHit)> {
    ray_cast.cast_ray(ray, &RayCastSettings::default()).first()
}
