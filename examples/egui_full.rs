//! Full egui example with egui_dock

use std::collections::HashMap;

use bevy::{
    prelude::*, render::camera::Viewport, window::PrimaryWindow,
    winit::WinitSettings,
};
use bevy_blendy_cameras::{
    BlendyCamerasPlugin, FlyCameraController, OrbitCameraController,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_dock::{DockArea, DockState, NodeIndex, Style};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(BlendyCamerasPlugin)
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(UiState::new())
        .add_systems(Startup, setup_system)
        .add_systems(Update, gui_system)
        .add_systems(Update, set_cameras_viewports_system.after(gui_system));
    app.run();
}

#[derive(Component)]
struct ViewportCamera(u32);

#[derive(Debug)]
enum DockTab {
    View3D(u32),
    Other,
}

#[derive(Resource)]
struct UiState {
    dock_state: DockState<DockTab>,
    viewport_rects: HashMap<u32, Option<egui::Rect>>,
}

impl UiState {
    fn new() -> Self {
        let mut state = DockState::new(vec![DockTab::Other]);
        let tree = state.main_surface_mut();
        let [_other, first_v3d] =
            tree.split_right(NodeIndex::root(), 0.2, vec![DockTab::View3D(0)]);
        let [_other, _second_v3d] =
            tree.split_right(first_v3d, 0.5, vec![DockTab::View3D(1)]);
        Self {
            dock_state: state,
            viewport_rects: HashMap::from([(0, None), (1, None)]),
        }
    }

    fn ui(&mut self, ctx: &mut egui::Context) {
        // Reset viewports rects to None in case one viewport is not visible
        self.viewport_rects
            .values_mut()
            .map(|viewport_rect| *viewport_rect = None)
            .count();
        let mut tab_viewer = TabViewer {
            viewport_rects: &mut self.viewport_rects,
        };
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

struct TabViewer<'a> {
    viewport_rects: &'a mut HashMap<u32, Option<egui::Rect>>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = DockTab;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, tab: &mut Self::Tab) {
        match tab {
            DockTab::View3D(n) => {
                self.viewport_rects.insert(*n, Some(ui.clip_rect()));
                // TODO:
            }
            DockTab::Other => {
                // TODO:
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
            DockTab::View3D(n) => format!("3D View {}", n).into(),
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

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        // TODO: Remove rect, camera, ...
        true
    }
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        ..default()
    });
    // Cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // Camera viewport 0
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        OrbitCameraController::default(),
        FlyCameraController {
            is_enabled: false,
            ..default()
        },
        ViewportCamera(0),
    ));
    // Camera viewport 1
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            camera: Camera {
                order: 1,
                // Clear on the second camera because the first camera might
                // not be visible and did not cleared the window
                //clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        OrbitCameraController::default(),
        FlyCameraController {
            is_enabled: false,
            ..default()
        },
        ViewportCamera(1),
    ));
}

fn gui_system(mut contexts: EguiContexts, mut ui_state: ResMut<UiState>) {
    let egui_context = contexts.ctx_mut();
    ui_state.ui(egui_context);
}

fn set_cameras_viewports_system(
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<(&mut Camera, &ViewportCamera)>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };
    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    for (mut cam, viewport_camera) in &mut cameras {
        let viewport_rect =
            ui_state.viewport_rects.get(&viewport_camera.0).unwrap();
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
