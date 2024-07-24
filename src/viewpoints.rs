use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::{
    // ActiveCameraData,
    fly::FlyCameraController,
    orbit::OrbitCameraController,
    utils,
};

#[derive(Debug, Copy, Clone)]
pub enum Viewpoint {
    Top,
    Bottom,
    Front,
    Back,
    Left,
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

    pub fn to_yaw_pitch(self) -> (f32, f32) {
        match self {
            Self::Top => (0.0, FRAC_PI_2),
            Self::Bottom => (0.0, -FRAC_PI_2),
            Self::Front => (0.0, 0.0),
            Self::Back => (PI, 0.0),
            Self::Left => (-FRAC_PI_2, 0.0),
            Self::Right => (FRAC_PI_2, 0.0),
        }
    }

    pub fn from_yaw_pitch(yaw: f32, pitch: f32) -> Option<Self> {
        // println!("{yaw} {pitch}");
        if utils::approx_equal(yaw, 0.0)
            && utils::approx_equal(pitch, FRAC_PI_2)
        {
            Some(Self::Top)
        } else if utils::approx_equal(yaw, 0.0)
            && utils::approx_equal(pitch, -FRAC_PI_2)
        {
            Some(Self::Bottom)
        } else if utils::approx_equal(yaw, 0.0)
            && utils::approx_equal(pitch, 0.0)
        {
            Some(Self::Front)
        } else if (utils::approx_equal(yaw, PI)
            || utils::approx_equal(yaw, -PI))
            && utils::approx_equal(pitch, 0.0)
        {
            Some(Self::Back)
        } else if utils::approx_equal(yaw, -FRAC_PI_2)
            && utils::approx_equal(pitch, 0.0)
        {
            Some(Self::Left)
        } else if utils::approx_equal(yaw, FRAC_PI_2)
            && utils::approx_equal(pitch, 0.0)
        {
            Some(Self::Right)
        } else {
            None
        }
    }

    pub fn from_transform(transform: &Transform) -> Option<Self> {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        Self::from_yaw_pitch(yaw, -pitch)
    }
}

#[derive(Event)]
pub struct ViewpointEvent(pub Viewpoint);

pub fn viewpoint_system(
    mut ev_read: EventReader<ViewpointEvent>,
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
    for ev in ev_read.read() {
        let (yaw, pitch) = ev.0.to_yaw_pitch();
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
        }
    }
}
