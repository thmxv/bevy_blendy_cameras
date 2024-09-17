use bevy::prelude::*;
use bevy_mod_raycast::deferred::RaycastMesh;

#[derive(Reflect)]
pub(crate) struct BlendyCamerasRaycastSet;

/// Make all the entites with a mesh a raycast target
pub(crate) fn add_to_raycast_system(
    mut commands: Commands,
    query: Query<Entity, Added<Handle<Mesh>>>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(RaycastMesh::<BlendyCamerasRaycastSet>::default());
    }
}

pub(crate) fn remove_from_raycast_system(
    mut commands: Commands,
    mut removals: RemovedComponents<Handle<Mesh>>,
) {
    for entity in removals.read() {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.remove::<RaycastMesh<BlendyCamerasRaycastSet>>();
        }
    }
}
