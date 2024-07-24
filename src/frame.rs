use bevy::{prelude::*, render::primitives::Aabb};

use crate::{fly::FlyCameraController, orbit::OrbitCameraController, utils};

#[derive(Event)]
pub struct FrameEvent {
    pub entities: Vec<Entity>,
    pub include_children: bool,
}

/// Return (min, max). If min > max there was no valid bounds to return.
fn get_entities_aabb(
    entities: &[Entity],
    include_children: bool,
    entities_query: &Query<
        (&GlobalTransform, Option<&Aabb>, Option<&Children>),
        (Without<OrbitCameraController>, Without<FlyCameraController>),
    >,
) -> (Vec3, Vec3) {
    let combine_bounds =
        |(a_min, a_max): (Vec3, Vec3), (b_min, b_max): (Vec3, Vec3)| {
            (a_min.min(b_min), a_max.max(b_max))
        };
    let default_bounds = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    entities
        .iter()
        .filter_map(|&entity| {
            entities_query
                .get(entity)
                .map(|(&tf, bounds, children)| {
                    let mut entity_bounds =
                        bounds.map_or(default_bounds, |bounds| {
                            (
                                tf * Vec3::from(bounds.min()),
                                tf * Vec3::from(bounds.max()),
                            )
                        });
                    if include_children {
                        if let Some(children) = children {
                            let children_bounds = get_entities_aabb(
                                children,
                                include_children,
                                entities_query,
                            );
                            entity_bounds =
                                combine_bounds(entity_bounds, children_bounds);
                        }
                    }
                    entity_bounds
                })
                .ok()
        })
        .fold(default_bounds, combine_bounds)
}

pub fn frame_system(
    mut ev_read: EventReader<FrameEvent>,
    // active_cam: Res<ActiveCameraData>,
    mut cameras_query: Query<
        (
            // Entity,
            &mut Transform,
            Option<&mut OrbitCameraController>,
            Option<&mut FlyCameraController>,
            &mut Projection,
        ),
        Or<(With<OrbitCameraController>, With<FlyCameraController>)>,
    >,
    entities_query: Query<
        (&GlobalTransform, Option<&Aabb>, Option<&Children>),
        (Without<OrbitCameraController>, Without<FlyCameraController>),
    >,
) {
    for ev in ev_read.read() {
        let FrameEvent {
            entities,
            include_children,
        } = ev;
        let (bounds_min, bounds_max) =
            get_entities_aabb(entities, *include_children, &entities_query);
        let aabb_diag = bounds_max - bounds_min;
        let aabb_diag = if aabb_diag.max_element() > 0.0 {
            aabb_diag
        } else {
            warn!(
                "Could not focus because entities (and children) do not \
                  have any AABB"
            );
            continue;
        };
        let aabb_center = bounds_min + aabb_diag * 0.5;
        let aabb_radius = aabb_diag.length();
        // TODO: Calculate distance acording to view angle (if projection is
        // perspective). Also (in perspective) center on the projection of
        // the object. For the moment we center on the AABB center but the
        // object is not centered in the view if viewed diagonaly.
        // For the moment just multiply distance to center to make sure all the
        // object in into view.
        let distance_camera_to_aabb_center = 1.2 * aabb_radius;
        let distance_camera_to_aabb_center =
            distance_camera_to_aabb_center.max(0.05);

        for (
            // entity,
            mut transform,
            orbit_controller_opt,
            fly_controller_opt,
            mut projection,
        ) in cameras_query.iter_mut()
        {
            if let Some(mut controller) = orbit_controller_opt {
                // NOTE: Checking if viewport is active does not work if
                // no manual manipulation of the camera is done a priory.

                // if controller.is_enabled && active_cam.entity == Some(entity) {
                if controller.is_enabled {
                    controller.focus = aabb_center;
                    controller.radius = Some(distance_camera_to_aabb_center);
                    controller.initialize_if_necessary(
                        &mut transform,
                        &mut projection,
                    );
                    utils::update_orbit_transform(
                        controller.yaw.unwrap(),
                        controller.pitch.unwrap(),
                        controller.radius.unwrap(),
                        controller.focus,
                        &mut transform,
                        &mut projection,
                    );
                }
            }
            if let Some(controller) = fly_controller_opt {
                // if controller.is_enabled && active_cam.entity == Some(entity) {
                if controller.is_enabled {
                    transform.translation = aabb_center
                        + (transform.back() * distance_camera_to_aabb_center);
                }
            }
        }
    }
}
