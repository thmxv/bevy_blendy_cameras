//! A example showing all the functionalities in multiple viewports

use std::collections::HashMap;

use bevy::{
    camera::Viewport,
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

use bevy_blendy_cameras::{
    get_camera_entity_from_cursor_position, BlendyCamerasPlugin,
    FlyCameraController, FrameEvent, OrbitCameraController, SwitchProjection,
    SwitchToFlyController, SwitchToOrbitController, Viewpoint, ViewpointEvent,
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

#[derive(Resource)]
struct Scene {
    pub scene_entity: Entity,
    pub cube_entity: Entity,
}

#[derive(Resource, Default)]
struct HelpText {
    pub camera_to_text_map: HashMap<Entity, Entity>,
}

#[derive(Component)]
struct CameraPosition {
    pos: UVec2,
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
                set_camera_viewports_system,
                switch_camera_controler_system,
                switch_camera_projection_system,
                switch_camera_viewpoint_system,
                frame_camera_system,
            ),
        )
        .run();
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Scene
    let mut cube_entity = Entity::PLACEHOLDER;
    let scene_entity = commands
        .spawn((Transform::default(), Visibility::default()))
        .with_children(|parent| {
            // Ground
            parent.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
            ));
            // Cube
            cube_entity = parent
                .spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
                    Transform::from_xyz(0.0, 0.5, 0.0),
                ))
                .id();
        })
        .id();
    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // Cameras
    let mut help_text_resource = HelpText::default();
    for n in 0..4 {
        let camera_entity = commands
            .spawn((
                CameraPosition {
                    pos: UVec2::new((n % 2) as u32, (n / 2) as u32),
                },
                Camera3d::default(),
                Camera {
                    order: n,
                    clear_color: if n > 0 {
                        ClearColorConfig::None
                    } else {
                        ClearColorConfig::default()
                    },
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
                OrbitCameraController::default(),
                FlyCameraController {
                    is_enabled: false,
                    ..default()
                },
            ))
            .id();
        // Help text
        let help_text_entity = commands
            .spawn((
                UiTargetCamera(camera_entity),
                Text::new(format!("{GENERAL_HELP_TEXT}\n{ORBIT_HELP_TEXT}")),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
            ))
            .id();
        help_text_resource
            .camera_to_text_map
            .insert(camera_entity, help_text_entity);
    }
    // Resources
    commands.insert_resource(Scene {
        scene_entity,
        cube_entity,
    });
    commands.insert_resource(help_text_resource);
}

fn set_camera_viewports_system(
    windows: Query<&Window>,
    mut resize_events: MessageReader<WindowResized>,
    mut query: Query<(&CameraPosition, &mut Camera)>,
) {
    // We need to dynamically resize the camera's viewports whenever the
    // window size changes so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to
    // reuse this system for initial setup.
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let size = window.physical_size() / 2;

        for (camera_position, mut camera) in &mut query {
            camera.viewport = Some(Viewport {
                physical_position: camera_position.pos * size,
                physical_size: size,
                ..default()
            });
        }
    }
}

// FIXME: Use the same event with parameter to switch
fn switch_camera_controler_system(
    mut commands: Commands,
    key_input: Res<ButtonInput<KeyCode>>,
    mut orbit_ev_writer: MessageWriter<SwitchToOrbitController>,
    mut fly_ev_writer: MessageWriter<SwitchToFlyController>,
    mut help_text: ResMut<HelpText>,
    mut cameras_query: Query<(Entity, &Camera), With<Camera3d>>,
    primary_window: Query<(Entity, &Window), With<PrimaryWindow>>,
    other_windows: Query<(Entity, &Window), Without<PrimaryWindow>>,
) {
    if key_input.just_pressed(KeyCode::KeyF) {
        let camera_entity = get_camera_entity_from_cursor_position(
            &cameras_query.transmute_lens::<(Entity, &Camera)>().query(),
            &primary_window,
            &other_windows,
        );
        if let Some(camera_entity) = camera_entity {
            fly_ev_writer.write(SwitchToFlyController { camera_entity });
            change_help_text(
                format!("{GENERAL_HELP_TEXT}\n{FLY_HELP_TEXT}"),
                &mut commands,
                &mut help_text,
                camera_entity,
            );
        }
    }
    if key_input.just_pressed(KeyCode::KeyO) {
        let camera_entity = get_camera_entity_from_cursor_position(
            &cameras_query.transmute_lens::<(Entity, &Camera)>().query(),
            &primary_window,
            &other_windows,
        );
        if let Some(camera_entity) = camera_entity {
            orbit_ev_writer.write(SwitchToOrbitController { camera_entity });
            change_help_text(
                format!("{GENERAL_HELP_TEXT}\n{ORBIT_HELP_TEXT}"),
                &mut commands,
                &mut help_text,
                camera_entity,
            );
        }
    }
}

fn switch_camera_projection_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut ev_writer: MessageWriter<SwitchProjection>,
    mut cameras_query: Query<(Entity, &Camera), With<Camera3d>>,
    primary_window: Query<(Entity, &Window), With<PrimaryWindow>>,
    other_windows: Query<(Entity, &Window), Without<PrimaryWindow>>,
) {
    if key_input.just_pressed(KeyCode::Numpad5) {
        let camera_entity = get_camera_entity_from_cursor_position(
            &cameras_query.transmute_lens::<(Entity, &Camera)>().query(),
            &primary_window,
            &other_windows,
        );
        if let Some(camera_entity) = camera_entity {
            ev_writer.write(SwitchProjection { camera_entity });
        }
    }
}

fn switch_camera_viewpoint_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut ev_writer: MessageWriter<ViewpointEvent>,
    mut cameras_query: Query<(Entity, &Camera), With<Camera3d>>,
    primary_window: Query<(Entity, &Window), With<PrimaryWindow>>,
    other_windows: Query<(Entity, &Window), Without<PrimaryWindow>>,
) {
    if !key_input.pressed(KeyCode::ShiftLeft)
        && !key_input.pressed(KeyCode::ShiftRight)
        && key_input.pressed(KeyCode::Numpad1)
    {
        let camera_entity = get_camera_entity_from_cursor_position(
            &cameras_query.transmute_lens::<(Entity, &Camera)>().query(),
            &primary_window,
            &other_windows,
        );
        if let Some(camera_entity) = camera_entity {
            ev_writer.write(ViewpointEvent {
                camera_entity,
                viewpoint: Viewpoint::Front,
            });
        }
    }
    if (key_input.pressed(KeyCode::ShiftLeft)
        || key_input.pressed(KeyCode::ShiftRight))
        && key_input.pressed(KeyCode::Numpad1)
    {
        let camera_entity = get_camera_entity_from_cursor_position(
            &cameras_query.transmute_lens::<(Entity, &Camera)>().query(),
            &primary_window,
            &other_windows,
        );
        if let Some(camera_entity) = camera_entity {
            ev_writer.write(ViewpointEvent {
                camera_entity,
                viewpoint: Viewpoint::Back,
            });
        }
    }
    if !key_input.pressed(KeyCode::ShiftLeft)
        && !key_input.pressed(KeyCode::ShiftRight)
        && key_input.pressed(KeyCode::Numpad3)
    {
        let camera_entity = get_camera_entity_from_cursor_position(
            &cameras_query.transmute_lens::<(Entity, &Camera)>().query(),
            &primary_window,
            &other_windows,
        );
        if let Some(camera_entity) = camera_entity {
            ev_writer.write(ViewpointEvent {
                camera_entity,
                viewpoint: Viewpoint::Right,
            });
        }
    }
    if (key_input.pressed(KeyCode::ShiftLeft)
        || key_input.pressed(KeyCode::ShiftRight))
        && key_input.pressed(KeyCode::Numpad3)
    {
        let camera_entity = get_camera_entity_from_cursor_position(
            &cameras_query.transmute_lens::<(Entity, &Camera)>().query(),
            &primary_window,
            &other_windows,
        );
        if let Some(camera_entity) = camera_entity {
            ev_writer.write(ViewpointEvent {
                camera_entity,
                viewpoint: Viewpoint::Left,
            });
        }
    }
    if !key_input.pressed(KeyCode::ShiftLeft)
        && !key_input.pressed(KeyCode::ShiftRight)
        && key_input.pressed(KeyCode::Numpad7)
    {
        let camera_entity = get_camera_entity_from_cursor_position(
            &cameras_query.transmute_lens::<(Entity, &Camera)>().query(),
            &primary_window,
            &other_windows,
        );
        if let Some(camera_entity) = camera_entity {
            ev_writer.write(ViewpointEvent {
                camera_entity,
                viewpoint: Viewpoint::Top,
            });
        }
    }
    if (key_input.pressed(KeyCode::ShiftLeft)
        || key_input.pressed(KeyCode::ShiftRight))
        && key_input.pressed(KeyCode::Numpad7)
    {
        let camera_entity = get_camera_entity_from_cursor_position(
            &cameras_query.transmute_lens::<(Entity, &Camera)>().query(),
            &primary_window,
            &other_windows,
        );
        if let Some(camera_entity) = camera_entity {
            ev_writer.write(ViewpointEvent {
                camera_entity,
                viewpoint: Viewpoint::Bottom,
            });
        }
    }
}

fn frame_camera_system(
    mut ev_reader: MessageReader<KeyboardInput>,
    mut ev_writer: MessageWriter<FrameEvent>,
    scene: Res<Scene>,
    mut cameras_query: Query<(Entity, &Camera), With<Camera3d>>,
    primary_window: Query<(Entity, &Window), With<PrimaryWindow>>,
    other_windows: Query<(Entity, &Window), Without<PrimaryWindow>>,
) {
    for ev in ev_reader.read() {
        if ev.state == ButtonState::Pressed {
            match &ev.logical_key {
                Key::Home => {
                    let camera_entity = get_camera_entity_from_cursor_position(
                        &cameras_query
                            .transmute_lens::<(Entity, &Camera)>()
                            .query(),
                        &primary_window,
                        &other_windows,
                    );
                    if let Some(camera_entity) = camera_entity {
                        ev_writer.write(FrameEvent {
                            camera_entity,
                            entities_to_be_framed: vec![scene.scene_entity],
                            include_children: true,
                        });
                    }
                }
                Key::Character(str) => {
                    if str == "c" {
                        let camera_entity =
                            get_camera_entity_from_cursor_position(
                                &cameras_query
                                    .transmute_lens::<(Entity, &Camera)>()
                                    .query(),
                                &primary_window,
                                &other_windows,
                            );
                        if let Some(camera_entity) = camera_entity {
                            ev_writer.write(FrameEvent {
                                camera_entity,
                                entities_to_be_framed: vec![scene.cube_entity],
                                include_children: false,
                            });
                        }
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
    camera_entity: Entity,
) {
    commands
        .entity(*help_text.camera_to_text_map.get(&camera_entity).unwrap())
        .despawn();
    let help_text_entity = commands
        .spawn((
            UiTargetCamera(camera_entity),
            Text::new(text),
            TextFont {
                font_size: 14.0,
                ..default()
            },
        ))
        .id();
    help_text
        .camera_to_text_map
        .insert(camera_entity, help_text_entity);
}
