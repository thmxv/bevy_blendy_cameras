//! Full egui example with egui_dock

use std::collections::HashMap;

use bevy::{
    ecs::{
        schedule::{LogLevel, ScheduleBuildSettings},
        system::SystemState,
    },
    prelude::*,
    render::camera::Viewport,
    window::PrimaryWindow,
    winit::WinitSettings,
};
use bevy_blendy_cameras::{
    BlendyCamerasPlugin, FlyCameraController, FrameEvent,
    OrbitCameraController, SwitchProjection, SwitchToFlyController,
    SwitchToOrbitController, Viewpoint, ViewpointEvent,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};

fn main() {
    let mut app = App::new();
    app.configure_schedules(ScheduleBuildSettings {
        ambiguity_detection: LogLevel::Warn,
        ..default()
    });
    app.add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(BlendyCamerasPlugin)
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup_system)
        .add_systems(
            Update,
            (gui_system_exclusive, set_cameras_viewports_system).chain(),
        );
    app.run();
}

#[derive(Debug)]
enum DockTab {
    View3D(Entity),
    Other,
}

#[derive(Resource)]
struct Scene {
    scene_entity: Entity,
    cube_entity: Entity,
}

#[derive(Resource)]
struct UiState {
    dock_state: DockState<DockTab>,
    camera_viewport_rects_map: HashMap<Entity, Option<egui::Rect>>,
    order_counter: isize,
}

impl UiState {
    fn new(camera_entities: &Vec<Entity>) -> Self {
        let mut dock_state = DockState::new(vec![DockTab::Other]);
        let tree = dock_state.main_surface_mut();
        let mut camera_viewport_rects_map: HashMap<Entity, Option<egui::Rect>> =
            HashMap::new();
        let mut to_split = NodeIndex::root();
        let order_counter = camera_entities
            .iter()
            .enumerate()
            .map(|(n, &camera_entity)| {
                let [_other, new] = tree.split_right(
                    to_split,
                    if n > 0 { 0.5 } else { 0.2 },
                    vec![DockTab::View3D(camera_entity)],
                );
                to_split = new;
                camera_viewport_rects_map.insert(camera_entity, None);
            })
            .count();
        Self {
            dock_state,
            camera_viewport_rects_map,
            order_counter: order_counter as isize,
        }
    }

    fn ui(&mut self, ctx: &mut egui::Context, world: &mut World) {
        // Reset viewports rects to None in case one viewport is not visible
        self.camera_viewport_rects_map
            .values_mut()
            .map(|viewport_rect| *viewport_rect = None)
            .count();
        let mut added_tabs = Vec::new();
        let mut tab_viewer = TabViewer {
            viewport_rects: &mut self.camera_viewport_rects_map,
            world,
            added_tabs: &mut added_tabs,
        };
        let mut style = Style::from_egui(ctx.style().as_ref());
        style.tab.tab_body.inner_margin = egui_dock::egui::Margin::same(0.0);
        DockArea::new(&mut self.dock_state)
            .show_add_buttons(true)
            .show_add_popup(true)
            .style(style)
            .show(ctx, &mut tab_viewer);
        added_tabs.drain(..).for_each(|(tab, surface, node)| {
            let tab = match tab {
                DockTab::View3D(_) => {
                    let camera_entity = world
                        .spawn((
                            Camera3dBundle {
                                transform: Transform::from_translation(
                                    Vec3::new(0.0, 1.5, 5.0),
                                ),
                                camera: Camera {
                                    order: self.order_counter,
                                    ..default()
                                },
                                ..default()
                            },
                            OrbitCameraController::default(),
                            FlyCameraController {
                                is_enabled: false,
                                ..default()
                            },
                        ))
                        .id();
                    self.order_counter += 1;
                    self.camera_viewport_rects_map.insert(camera_entity, None);
                    DockTab::View3D(camera_entity)
                }
                _ => tab,
            };
            self.dock_state
                .set_focused_node_and_surface((surface, node));
            self.dock_state.push_to_focused_leaf(tab);
        });
    }
}

struct TabViewer<'a> {
    viewport_rects: &'a mut HashMap<Entity, Option<egui::Rect>>,
    world: &'a mut World,
    added_tabs: &'a mut Vec<(DockTab, SurfaceIndex, NodeIndex)>,
}

impl TabViewer<'_> {
    fn view3d_menu_ui(
        &mut self,
        ui: &mut egui_dock::egui::Ui,
        camera_entity: Entity,
    ) {
        egui::Frame::default()
            .fill(ui.visuals().window_fill)
            .inner_margin(egui::Margin::symmetric(8.0, 2.0))
            .show(ui, |ui| {
                egui::menu::bar(ui, |ui| {
                    egui::menu::menu_button(ui, "View", |ui| {
                        if ui.button("Frame All").clicked() {
                            let scene = self.world.resource::<Scene>();
                            self.world.send_event(FrameEvent {
                                camera_entity,
                                entities_to_be_framed: vec![scene.scene_entity],
                                include_children: true,
                            });
                            ui.close_menu();
                        }
                        if ui.button("Frame Cube").clicked() {
                            let scene = self.world.resource::<Scene>();
                            self.world.send_event(FrameEvent {
                                camera_entity,
                                entities_to_be_framed: vec![scene.cube_entity],
                                include_children: false,
                            });
                            ui.close_menu();
                        }
                        if ui.button("Perspective/Orthographic").clicked() {
                            self.world
                                .send_event(SwitchProjection { camera_entity });
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.menu_button("Viewpoint", |ui| {
                            if ui.button("Top").clicked() {
                                self.world.send_event(ViewpointEvent {
                                    camera_entity,
                                    viewpoint: Viewpoint::Top,
                                });
                                ui.close_menu();
                            }
                            if ui.button("Bottom").clicked() {
                                self.world.send_event(ViewpointEvent {
                                    camera_entity,
                                    viewpoint: Viewpoint::Bottom,
                                });
                                ui.close_menu();
                            }
                            if ui.button("Front").clicked() {
                                self.world.send_event(ViewpointEvent {
                                    camera_entity,
                                    viewpoint: Viewpoint::Front,
                                });
                                ui.close_menu();
                            }
                            if ui.button("Back").clicked() {
                                self.world.send_event(ViewpointEvent {
                                    camera_entity,
                                    viewpoint: Viewpoint::Back,
                                });
                                ui.close_menu();
                            }
                            if ui.button("Left").clicked() {
                                self.world.send_event(ViewpointEvent {
                                    camera_entity,
                                    viewpoint: Viewpoint::Left,
                                });
                                ui.close_menu();
                            }
                            if ui.button("Right").clicked() {
                                self.world.send_event(ViewpointEvent {
                                    camera_entity,
                                    viewpoint: Viewpoint::Right,
                                });
                                ui.close_menu();
                            }
                        });
                        ui.menu_button("Navigation", |ui| {
                            if ui.button("Orbit").clicked() {
                                self.world.send_event(
                                    SwitchToOrbitController { camera_entity },
                                );
                                ui.close_menu();
                            }
                            if ui.button("Fly").clicked() {
                                self.world.send_event(SwitchToFlyController {
                                    camera_entity,
                                });
                                ui.close_menu();
                            }
                        });
                    });
                });
            });
    }

    fn view3d_toolbar_ui(
        &mut self,
        ui: &mut egui_dock::egui::Ui,
        camera_entity: Entity,
    ) -> egui_dock::egui::Rect {
        let mut system_state: SystemState<
            Query<(&OrbitCameraController, &FlyCameraController)>,
        > = SystemState::new(self.world);
        let cameras_query = system_state.get_mut(self.world);
        let mut switch_to_orbit = false;
        let mut switch_to_fly = false;
        let margin = ui.style().spacing.window_margin.left;
        let item_spacing = ui.style().spacing.item_spacing;
        let viewport_rect =
            self.viewport_rects.get(&camera_entity).unwrap().unwrap();
        let offset = viewport_rect.left_top() + item_spacing;
        let response = egui::Area::new(egui::Id::new(format!(
            "toolbar_area{}",
            camera_entity,
        )))
        .anchor(egui::Align2::LEFT_TOP, offset.to_vec2())
        .show(ui.ctx(), |ui| {
            ui.set_clip_rect(viewport_rect);
            egui::Frame::none()
                .fill(ui.visuals().window_fill)
                .inner_margin(margin)
                .show(ui, |ui| {
                    let (orbit_controller, fly_controller) =
                        cameras_query.get(camera_entity).unwrap();
                    let button = egui::Button::new("Orbit")
                        .selected(orbit_controller.is_enabled);
                    if ui.add(button).clicked() {
                        switch_to_orbit = true;
                    }
                    let button = egui::Button::new("Fly")
                        .selected(fly_controller.is_enabled);
                    if ui.add(button).clicked() {
                        switch_to_fly = true;
                    }
                });
        });
        if switch_to_orbit {
            self.world
                .send_event(SwitchToOrbitController { camera_entity });
        }
        if switch_to_fly {
            self.world
                .send_event(SwitchToFlyController { camera_entity });
        }
        response.response.rect
    }

    fn view3d_stats(
        &mut self,
        ui: &mut egui_dock::egui::Ui,
        camera_entity: Entity,
    ) {
        let mut system_state: SystemState<
            Query<(&Transform, &Projection), With<Camera3d>>,
        > = SystemState::new(self.world);
        let camera_query = system_state.get(self.world);
        let (transform, projection) = camera_query.get(camera_entity).unwrap();
        let viewpoint_text = match Viewpoint::from_transform(transform) {
            Viewpoint::User { yaw: _, pitch: _ } => "User".to_string(),
            vp => format!("{vp:?}"),
        };
        let projection_text = match *projection {
            Projection::Orthographic(_) => "Orthographic",
            Projection::Perspective(_) => "Perspective",
        };
        let text =
            egui::RichText::new(format!("{viewpoint_text} {projection_text}"))
                .color(egui::Color32::WHITE);
        ui.add(egui::Label::new(text));
    }

    fn view3d_axes(
        &mut self,
        ui: &mut egui_dock::egui::Ui,
        camera_entity: Entity,
    ) {
        // 64 lines, 12 text, 24 margin
        let desired_size = egui::Vec2::splat(64.0 + 12.0 + 24.0);
        let (_id, rect) = ui.allocate_space(desired_size);
        if ui.is_rect_visible(rect) {
            let mut system_state: SystemState<
                Query<&Transform, With<Camera3d>>,
            > = SystemState::new(self.world);
            let camera_query = system_state.get(self.world);
            let rotation = camera_query.get(camera_entity).unwrap().rotation;
            let rotation = rotation.inverse();
            let start_point = rect.center();
            for (label, direction, color) in [
                ("x", Vec3::X, (0.0, 0.77, 0.67, 1.0)),
                ("y", Vec3::Y, (0.25, 0.90, 0.68, 1.0)),
                ("z", Vec3::Z, (0.60, 0.90, 0.80, 1.0)),
            ] {
                let axis = rotation.mul_vec3(direction * 32.0);
                let axis = egui::Vec2::new(axis.x, -axis.y);
                let end_point = start_point + axis;
                let color =
                    egui::ecolor::Hsva::new(color.0, color.1, color.2, 1.0)
                        .into();
                ui.painter().line_segment(
                    [start_point, end_point],
                    egui::Stroke { width: 1.0, color },
                );
                ui.painter().text(
                    end_point,
                    egui::Align2::LEFT_BOTTOM,
                    label,
                    egui::FontId::monospace(12.0),
                    color,
                );
            }
        }
    }
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = DockTab;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, tab: &mut Self::Tab) {
        match tab {
            DockTab::View3D(camera_entity) => {
                self.view3d_menu_ui(ui, *camera_entity);
                self.viewport_rects.insert(
                    *camera_entity,
                    Some(ui.available_rect_before_wrap()),
                    //Some(ui.clip_rect()),
                );
                let toolbar_rect = self.view3d_toolbar_ui(ui, *camera_entity);
                ui.horizontal(|ui| {
                    ui.add_space(toolbar_rect.width() + 12.0);
                    self.view3d_stats(ui, *camera_entity);
                    ui.with_layout(
                        egui::Layout::top_down(egui::Align::RIGHT),
                        |ui| {
                            self.view3d_axes(ui, *camera_entity);
                        },
                    );
                });
            }
            DockTab::Other => {
                ui.label(format!("Content of tab {tab:?}"));
            }
        }
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        match tab {
            DockTab::View3D(n) => format!("View3D{}", n).into(),
            _ => format!("{tab:?}").into(),
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        match tab {
            DockTab::View3D(_) => "3D View".into(),
            _ => format!("{tab:?}").into(),
        }
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        match tab {
            DockTab::View3D(_) => false,
            _ => true,
        }
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        match tab {
            DockTab::View3D(_) => false,
            _ => true,
        }
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            DockTab::View3D(camera_entity) => {
                self.viewport_rects.remove(camera_entity);
            }
            _ => {}
        }
        true
    }

    fn add_popup(
        &mut self,
        ui: &mut egui::Ui,
        surface: egui_dock::SurfaceIndex,
        node: NodeIndex,
    ) {
        ui.set_min_width(120.0);
        ui.style_mut().visuals.button_frame = false;
        if ui.button("View 3D").clicked() {
            self.added_tabs.push((
                DockTab::View3D(Entity::PLACEHOLDER),
                surface,
                node,
            ));
        }
        // Not for now because of ID conflicts
        //if ui.button("Other").clicked() {
        //    self.added_tabs.push((DockTab::Other, surface, node));
        //}
    }
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Scene
    let mut cube_entity = Entity::PLACEHOLDER;
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
            cube_entity = parent
                .spawn(PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                    material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..default()
                })
                .id();
        })
        .id();
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // Cameras
    let mut camera_entities = Vec::new();
    for n in 0..2 {
        let camera_entity = commands
            .spawn((
                Camera3dBundle {
                    transform: Transform::from_translation(Vec3::new(
                        0.0, 1.5, 5.0,
                    )),
                    camera: Camera {
                        order: n,
                        ..default()
                    },
                    ..default()
                },
                OrbitCameraController::default(),
                FlyCameraController {
                    is_enabled: false,
                    // Clear on the second camera because the first camera might
                    // not be visible and did not cleared the window
                    ..default()
                },
            ))
            .id();
        camera_entities.push(camera_entity);
    }
    // Resources
    commands.insert_resource(UiState::new(&camera_entities));
    commands.insert_resource(Scene {
        scene_entity,
        cube_entity,
    });
}

fn gui_system_exclusive(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope(|world, mut ui_state: Mut<UiState>| {
        ui_state.ui(egui_context.get_mut(), world);
    });
}

fn set_cameras_viewports_system(
    ui_state: Res<UiState>,
    primary_window: Query<
        (&mut Window, &bevy_egui::EguiSettings),
        With<PrimaryWindow>,
    >,
    mut cameras: Query<(Entity, &mut Camera)>,
) {
    let Ok((window, window_egui_settings)) = primary_window.get_single() else {
        return;
    };
    let scale_factor =
        window.scale_factor() * window_egui_settings.scale_factor;

    for (entity, mut cam) in &mut cameras {
        let viewport_rect =
            ui_state.camera_viewport_rects_map.get(&entity).unwrap();
        if let Some(viewport_rect) = viewport_rect {
            let viewport_pos =
                viewport_rect.left_top().to_vec2() * scale_factor;
            let viewport_size = viewport_rect.size() * scale_factor;

            let physical_position =
                UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32);
            let physical_size =
                UVec2::new(viewport_size.x as u32, viewport_size.y as u32);

            // The desired viewport rectangle at its offset in "physical pixel
            // space"
            let rect = physical_position + physical_size;

            let window_size = window.physical_size();
            // wgpu will panic if trying to set a viewport rect which has
            // coordinates extending past the size of the render target, i.e. the
            // physical window in our case. Typically this shouldn't happen- but
            // during init and resizing etc. edge cases might occur. Simply do
            // nothing in those cases.
            if rect.x <= window_size.x && rect.y <= window_size.y {
                cam.is_active = true;
                cam.viewport = Some(Viewport {
                    physical_position,
                    physical_size,
                    depth: 0.0..1.0,
                });
            }
        } else {
            cam.is_active = false;
            cam.viewport = Some(Viewport {
                physical_position: UVec2::ZERO,
                physical_size: UVec2::ZERO,
                depth: 0.0..1.0,
            });
        }
    }
}
