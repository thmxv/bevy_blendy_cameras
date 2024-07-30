use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
    //render::camera::ScalingMode,
};

use bevy_blendy_cameras::{
    viewpoints::Viewpoint, BlendyCamerasPlugin, FlyCameraController,
    FrameEvent, OrbitCameraController, SwitchProjection, SwitchToFlyController,
    SwitchToOrbitController, ViewpointEvent,
};

// FIXME: Make fly mode work in ortho projection
const GENERAL_HELP_TEXT: &str = "\
    Press F to switch to Fly camera controler\n\
    Press O to switch to Orbit camera controler\n\
    Press Numpad 5 to switch between orthographic/perspective \
    projections\n    (In Orbit mode only, Fly mode only works in \
    perspective)\n\
    Press Home to frame the whole scene\n\
    Press C to frame the cube\n\
    Press Numpad 1 to view from the front\n\
    Press Shift + Numpad 1 to view from the rear\n\
    Press Numpad 3 to view from the right\n\
    Press Shift + Numpad 1 to view from the left\n\
    Press Numpad 7 to view from the top\n\
    Press Shift + Numpad 7 to view from the bottom\n\
    ";

const ORBIT_HELP_TEXT: &str = "\
    Press Middle Mouse button and drag to orbit camera\n\
    Press Shift + Middle Mouse button and drag to pan camera\n\
    Scoll the mouse wheel to zoom\n\
    ";

const FLY_HELP_TEXT: &str = "\
    Press Middle Mouse button and drag to rotate camera\n\
    Scoll the mouse wheel to change de camera mouvement speed\n\
    Press W to pan down\n\
    Press R to pan up\n\
    Press E to zoom forward\n\
    Press D to zoom backward\n\
    Press S to pan left\n\
    Press F to pan right\n\
    ";

#[derive(Default, Resource)]
pub struct Scene {
    pub scene_entity: Option<Entity>,
    pub cube_entity: Option<Entity>,
}

#[derive(Default, Resource)]
pub struct HelpText {
    pub help_text_entity: Option<Entity>,
}

fn main() {
    let mut app = App::new();
    app.configure_schedules(ScheduleBuildSettings {
        ambiguity_detection: LogLevel::Warn,
        ..default()
    });
    app.add_plugins(DefaultPlugins)
        .add_plugins(BlendyCamerasPlugin)
        .add_systems(Startup, setup_system)
        .add_systems(
            Update,
            (
                switch_camera_controler_system,
                switch_camera_projection_system,
                switch_camera_viewpoint_system,
                frame_camera_system,
            ),
        )
        .insert_resource(Scene::default())
        .insert_resource(HelpText::default())
        .run();
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scene: ResMut<Scene>,
    mut help_text: ResMut<HelpText>,
) {
    // Scene
    let scene_entity = commands
        .spawn(SpatialBundle::default())
        .with_children(|parent| {
            // Ground
            parent.spawn(PbrBundle {
                mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
                material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
                ..default()
            });
            // Cube
            let cube_entity = parent
                .spawn(PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                    material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..default()
                })
                .id();
            scene.cube_entity = Some(cube_entity);
        })
        .id();
    scene.scene_entity = Some(scene_entity);
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // Camera
    commands.spawn((
        Camera3dBundle {
            //projection: Projection::Orthographic(OrthographicProjection {
            //    scaling_mode: ScalingMode::FixedVertical(1.0),
            //    ..default()
            //}),
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        OrbitCameraController::default(),
        FlyCameraController {
            is_enabled: false,
            ..default()
        },
    ));
    // Help text
    let help_text_entity = commands
        .spawn(TextBundle::from_section(
            format!("{GENERAL_HELP_TEXT}\n{ORBIT_HELP_TEXT}"),
            TextStyle::default(),
        ))
        .id();
    help_text.help_text_entity = Some(help_text_entity);
}

// FIXME: Use the same event with parameter to switch
fn switch_camera_controler_system(
    mut commands: Commands,
    key_input: Res<ButtonInput<KeyCode>>,
    mut orbit_ev_writer: EventWriter<SwitchToOrbitController>,
    mut fly_ev_writer: EventWriter<SwitchToFlyController>,
    mut help_text: ResMut<HelpText>,
) {
    if key_input.just_pressed(KeyCode::KeyF) {
        fly_ev_writer.send_default();
        change_help_text(
            format!("{GENERAL_HELP_TEXT}\n{FLY_HELP_TEXT}"),
            &mut commands,
            &mut help_text,
        );
    }
    if key_input.just_pressed(KeyCode::KeyO) {
        orbit_ev_writer.send_default();
        change_help_text(
            format!("{GENERAL_HELP_TEXT}\n{ORBIT_HELP_TEXT}"),
            &mut commands,
            &mut help_text,
        );
    }
}

fn switch_camera_projection_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut ev_writer: EventWriter<SwitchProjection>,
) {
    if key_input.just_pressed(KeyCode::Numpad5) {
        ev_writer.send_default();
    }
}

fn switch_camera_viewpoint_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut ev_writer: EventWriter<ViewpointEvent>,
) {
    if !key_input.pressed(KeyCode::ShiftLeft)
        && !key_input.pressed(KeyCode::ShiftRight)
        && key_input.pressed(KeyCode::Numpad1)
    {
        ev_writer.send(ViewpointEvent(Viewpoint::Front));
    }
    if (key_input.pressed(KeyCode::ShiftLeft)
        || key_input.pressed(KeyCode::ShiftRight))
        && key_input.pressed(KeyCode::Numpad1)
    {
        ev_writer.send(ViewpointEvent(Viewpoint::Back));
    }
    if !key_input.pressed(KeyCode::ShiftLeft)
        && !key_input.pressed(KeyCode::ShiftRight)
        && key_input.pressed(KeyCode::Numpad3)
    {
        ev_writer.send(ViewpointEvent(Viewpoint::Right));
    }
    if (key_input.pressed(KeyCode::ShiftLeft)
        || key_input.pressed(KeyCode::ShiftRight))
        && key_input.pressed(KeyCode::Numpad3)
    {
        ev_writer.send(ViewpointEvent(Viewpoint::Left));
    }
    if !key_input.pressed(KeyCode::ShiftLeft)
        && !key_input.pressed(KeyCode::ShiftRight)
        && key_input.pressed(KeyCode::Numpad7)
    {
        ev_writer.send(ViewpointEvent(Viewpoint::Top));
    }
    if (key_input.pressed(KeyCode::ShiftLeft)
        || key_input.pressed(KeyCode::ShiftRight))
        && key_input.pressed(KeyCode::Numpad7)
    {
        ev_writer.send(ViewpointEvent(Viewpoint::Bottom));
    }
}

fn frame_camera_system(
    mut ev_reader: EventReader<KeyboardInput>,
    mut ev_writer: EventWriter<FrameEvent>,
    scene: Res<Scene>,
) {
    for ev in ev_reader.read() {
        if ev.state == ButtonState::Pressed {
            match &ev.logical_key {
                Key::Home => {
                    ev_writer.send(FrameEvent {
                        entities: vec![scene.scene_entity.unwrap()],
                        include_children: true,
                    });
                }
                Key::Character(str) => {
                    if str == "c" {
                        ev_writer.send(FrameEvent {
                            entities: vec![scene.cube_entity.unwrap()],
                            include_children: false,
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

fn change_help_text(
    text: String,
    commands: &mut Commands,
    help_text: &mut HelpText,
) {
    commands
        .entity(help_text.help_text_entity.unwrap())
        .despawn_recursive();
    help_text.help_text_entity = None;
    let help_text_entity = commands
        .spawn(TextBundle::from_section(text, TextStyle::default()))
        .id();
    help_text.help_text_entity = Some(help_text_entity);
}
