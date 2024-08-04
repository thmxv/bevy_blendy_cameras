use bevy::prelude::*;
use bevy_mod_raycast::deferred::RaycastMesh;

#[derive(Reflect)]
pub(crate) struct BlendyCamerasRaycastSet;

/// Make all the entites with a mesh a raycast target
pub(crate) fn startup_system(world: &mut World) {
    world
        .register_component_hooks::<Handle<Mesh>>()
        .on_add(|mut world, entity, _component_id| {
            world
                .commands()
                .entity(entity)
                .insert(RaycastMesh::<BlendyCamerasRaycastSet>::default());
        })
        .on_remove(|mut world, entity, _component_id| {
            world
                .commands()
                .entity(entity)
                .remove::<RaycastMesh<BlendyCamerasRaycastSet>>();
        });
}

