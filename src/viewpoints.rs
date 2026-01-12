use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::{
    // ActiveCameraData,
    fly::FlyCameraController,
    orbit::OrbitCameraController,
    utils,
};

/// Point of view of a camera, looking in the oposite direction
#[derive(Debug, Copy, Clone)]
pub enum Viewpoint {
    /// Custom user viewpoint
    User {
        /// Rotation around the local vertical axis
        yaw: f32,
        /// Rotation around the local horizontal transverse axis
        pitch: f32,
    },
    /// View from top
    Top,
    /// View from bottom
    Bottom,
    /// View from front
    Front,
    /// View from back
    Back,
    /// View from left
    Left,
    /// View from right
    Right,
}

impl Viewpoint {
    // Not used at the moment
    // pub fn to_view_direction(self) -> Vec3 {
    //     match self {
    //         Self::Top => Vec3::new(0.0, -1.0, 0.0),
    //         Self::Bottom => Vec3::new(0.0, 1.0, 0.0),
    //         Self::Front => Vec3::new(0.0, 0.0, 1.0),
    //         Self::Back => Vec3::new(0.0, 0.0, -1.0),
    //         Self::Left => Vec3::new(1.0, 0.0, 0.0),
    //         Self::Right => Vec3::new(-1.0, 0.0, 0.0),
    //     }
    // }

    pub(crate) fn to_yaw_pitch(self) -> (f32, f32) {
        match self {
            Self::User { yaw, pitch } => (yaw, pitch),
            Self::Top => (0.0, FRAC_PI_2),
            Self::Bottom => (0.0, -FRAC_PI_2),
            Self::Front => (0.0, 0.0),
            Self::Back => (PI, 0.0),
            Self::Left => (-FRAC_PI_2, 0.0),
            Self::Right => (FRAC_PI_2, 0.0),
        }
    }

    fn from_yaw_pitch(yaw: f32, pitch: f32) -> Self {
        // println!("{yaw} {pitch}");
        if utils::approx_equal(yaw, 0.0)
            && utils::approx_equal(pitch, FRAC_PI_2)
        {
            Self::Top
        } else if utils::approx_equal(yaw, 0.0)
            && utils::approx_equal(pitch, -FRAC_PI_2)
        {
            Self::Bottom
        } else if utils::approx_equal(yaw, 0.0)
            && utils::approx_equal(pitch, 0.0)
        {
            Self::Front
        } else if (utils::approx_equal(yaw, PI)
            || utils::approx_equal(yaw, -PI))
            && utils::approx_equal(pitch, 0.0)
        {
            Self::Back
        } else if utils::approx_equal(yaw, -FRAC_PI_2)
            && utils::approx_equal(pitch, 0.0)
        {
            Self::Left
        } else if utils::approx_equal(yaw, FRAC_PI_2)
            && utils::approx_equal(pitch, 0.0)
        {
            Self::Right
        } else {
            Self::User { yaw, pitch }
        }
    }

    /// Calculate [`Viewpoint`] from camera [`Transform`]
    pub fn from_transform(transform: &Transform) -> Self {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        Self::from_yaw_pitch(yaw, -pitch)
    }
}

/// Message used to set the camera point of view
#[derive(Message)]
pub struct ViewpointEvent {
    /// The camera for wich to change viewpoint
    pub camera_entity: Entity,
    /// The viewpoint to apply to the camera
    pub viewpoint: Viewpoint,
}

#[allow(clippy::type_complexity)]
pub(crate) fn viewpoint_system(
    mut ev_read: MessageReader<ViewpointEvent>,
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
) {
    for ViewpointEvent {
        camera_entity,
        viewpoint,
    } in ev_read.read()
    {
        if let Ok((
            // entity,
            mut transform,
            orbit_controller_opt,
            fly_controller_opt,
            mut projection,
        )) = cameras_query.get_mut(*camera_entity)
        {
            let (yaw, pitch) = viewpoint.to_yaw_pitch();
            if let Some(mut controller) = orbit_controller_opt {
                // NOTE: Checking if viewport is active does not work if
                // no manual manipulation of the camera is done a priory.

                // if controller.is_enabled && active_cam.entity == Some(entity) {
                if controller.is_enabled {
                    controller.yaw = Some(yaw);
                    controller.pitch = Some(pitch);
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
                    let rotation =
                        Quat::from_euler(EulerRot::YXZ, yaw, -pitch, 0.0);
                    transform.rotation = rotation;
                }
            }
        } else {
            warn!("Camera not found while trying to set viewpoint");
        }
    }
}
