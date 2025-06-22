#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]

use avian2d::prelude::{Collider, Gravity, PhysicsLayer, RigidBody};
use bevy::DefaultPlugins;
use bevy::app::{App, FixedMainScheduleOrder};
use bevy::asset::AssetMetaCheck;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy::winit::WinitWindows;
use bevy_optix::camera::MainCamera;
use bevy_optix::pixel_perfect::CanvasDimensions;
use std::io::Cursor;
use winit::window::Icon;

mod fragments;
mod interactions;
mod loading;
mod menu;
mod player;
mod textbox;
#[allow(unused)]
mod world;

pub const WIDTH: f32 = 320.;
pub const HEIGHT: f32 = 180.;
pub const RESOLUTION_SCALE: f32 = 4.;

pub const TILE_SIZE: f32 = 16.;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // TODO: Rename
                    title: "Time Marches On".to_string(),
                    canvas: Some("#bevy".to_owned()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    resolution: WindowResolution::new(
                        WIDTH * RESOLUTION_SCALE,
                        HEIGHT * RESOLUTION_SCALE,
                    ),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
        bevy_tween::DefaultTweenPlugins,
        bevy_seedling::SeedlingPlugin::default(),
        bevy_enhanced_input::EnhancedInputPlugin,
        avian2d::PhysicsPlugins::new(Avian).with_length_unit(8.),
        bevy_optix::pixel_perfect::PixelPerfectPlugin(CanvasDimensions {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            pixel_scale: RESOLUTION_SCALE,
        }),
        bevy_optix::debug::DebugPlugin,
        bevy_optix::camera::CameraAnimationPlugin,
        bevy_pretty_text::PrettyTextPlugin,
        bevy_ldtk_scene::LdtkScenePlugin,
        world::TimeMarchesOnPlugin,
        //bevy_egui::EguiPlugin {
        //    enable_multipass_for_primary_context: true,
        //},
        //bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
    ))
    .add_plugins((
        loading::LoadingPlugin,
        menu::MenuPlugin,
        player::PlayerPlugin,
        textbox::TextboxPlugin,
        interactions::InteractionPlugin,
    ))
    .init_state::<GameState>()
    .init_schedule(Avian)
    .insert_resource(Gravity(Vec2::ZERO))
    .add_systems(Startup, set_window_icon)
    .add_systems(Update, add_tile_collision)
    .add_systems(OnEnter(GameState::Playing), load_ldtk);

    app.world_mut()
        .resource_mut::<FixedMainScheduleOrder>()
        .insert_after(FixedPostUpdate, Avian);

    #[cfg(debug_assertions)]
    app.add_systems(Update, close_on_escape);
    #[cfg(debug_assertions)]
    app.add_plugins(avian2d::debug_render::PhysicsDebugPlugin::new(Avian))
        .add_systems(Update, enable_avian_debug);

    #[cfg(not(debug_assertions))]
    app.insert_resource(ClearColor(Color::BLACK));
    #[cfg(debug_assertions)]
    app.insert_resource(ClearColor(Color::linear_rgb(1., 0., 1.)));

    app.run();
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    Menu,
    Playing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Avian;

#[derive(Default, Clone, Copy, PartialEq, Eq, PhysicsLayer)]
pub enum Layer {
    #[default]
    Default,
}

pub struct HexColor(pub u32);

impl Into<Color> for HexColor {
    fn into(self) -> Color {
        Color::srgb_u8(
            (self.0 >> 16) as u8 & 0xFF,
            (self.0 >> 8) as u8 & 0xFF,
            self.0 as u8,
        )
    }
}

fn load_ldtk(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut camera: Single<&mut Camera, With<MainCamera>>,
) {
    camera.clear_color = ClearColorConfig::Custom(HexColor(0x252525).into());
    commands.spawn((
        bevy_ldtk_scene::HotWorld(server.load("ldtk/time-marches-on.ldtk")),
        bevy_ldtk_scene::World(server.load("ldtk/time-marches-on.ron")),
        bevy_ldtk_scene::prelude::LevelLoader::levels(world::Level0),
    ));
}

pub fn add_tile_collision(
    mut commands: Commands,
    tiles: Query<(&Transform, &ChildOf, &world::Tile), Added<world::Tile>>,
    levels: Query<Entity>,
) {
    if tiles.is_empty() {
        return;
    }

    let mut cached_collider_positions = Vec::with_capacity(1024);
    let tile_size = TILE_SIZE;

    let offset = tile_size / 2.;
    for transform in tiles
        .iter()
        .filter(|(_, _, t)| matches!(t, world::Tile::Collision))
        .map(|(t, _, _)| t)
    {
        cached_collider_positions.push(Vec2::new(
            transform.translation.x + offset,
            transform.translation.y + offset,
        ));
    }

    if cached_collider_positions.is_empty() {
        return;
    }

    // FIXME: assumes that one level is loaded at a time!!
    let level = tiles
        .iter()
        .next()
        .map(|(_, p, _)| levels.get(p.parent()).unwrap())
        .unwrap();

    commands.entity(level).with_children(|level| {
        for (pos, collider) in
            build_colliders_from_vec2(cached_collider_positions, tile_size).into_iter()
        {
            level.spawn((
                Transform::from_translation((pos - Vec2::splat(tile_size / 2.)).extend(0.)),
                RigidBody::Static,
                collider,
            ));
            //num_colliders += 1;
        }
    });
}

fn build_colliders_from_vec2(mut positions: Vec<Vec2>, tile_size: f32) -> Vec<(Vec2, Collider)> {
    positions.sort_by(|a, b| {
        let y_cmp = a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal);
        if y_cmp == std::cmp::Ordering::Equal {
            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            y_cmp
        }
    });

    let mut rows = Vec::with_capacity(positions.len() / 2);
    let mut current_y = None;
    let mut current_xs = Vec::with_capacity(positions.len() / 2);
    for v in positions.into_iter() {
        match current_y {
            None => {
                current_y = Some(v.y);
                current_xs.push(v.x);
            }
            Some(y) => {
                if v.y == y {
                    current_xs.push(v.x);
                } else {
                    rows.push((y, current_xs.clone()));
                    current_xs.clear();

                    current_y = Some(v.y);
                    current_xs.push(v.x);
                }
            }
        }
    }

    match current_y {
        Some(y) => {
            rows.push((y, current_xs));
        }
        None => unreachable!(),
    }

    #[derive(Debug, Clone, Copy)]
    struct Plate {
        y: f32,
        x_start: f32,
        x_end: f32,
    }

    let mut row_plates = Vec::with_capacity(rows.len());
    for (y, row) in rows.into_iter() {
        let mut current_x = None;
        let mut x_start = None;
        let mut plates = Vec::with_capacity(row.len() / 4);

        for x in row.iter() {
            match (current_x, x_start) {
                (None, None) => {
                    current_x = Some(*x);
                    x_start = Some(*x);
                }
                (Some(cx), Some(xs)) => {
                    if *x > cx + tile_size {
                        plates.push(Plate {
                            x_end: cx + tile_size,
                            x_start: xs,
                            y,
                        });
                        x_start = Some(*x);
                    }

                    current_x = Some(*x);
                }
                _ => unreachable!(),
            }
        }

        match (current_x, x_start) {
            (Some(cx), Some(xs)) => {
                plates.push(Plate {
                    x_end: cx + tile_size,
                    x_start: xs,
                    y,
                });
            }
            _ => unreachable!(),
        }

        row_plates.push(plates);
    }

    let mut output = Vec::new();
    for plates in row_plates.into_iter() {
        for plate in plates.into_iter() {
            output.push((
                Vec2::new(
                    plate.x_end - (plate.x_end - plate.x_start) / 2.,
                    plate.y - tile_size / 2.,
                ),
                Collider::rectangle(plate.x_end - plate.x_start, tile_size),
            ));
        }
    }

    output
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) -> Result {
    let primary_entity = primary_window.single()?;
    let Some(primary) = windows.get_window(primary_entity) else {
        return Err(BevyError::from("No primary window!"));
    };
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };

    Ok(())
}

#[cfg(debug_assertions)]
fn close_on_escape(input: Res<ButtonInput<KeyCode>>, mut writer: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        writer.write(AppExit::Success);
    }
}

#[cfg(debug_assertions)]
fn enable_avian_debug(mut store: ResMut<GizmoConfigStore>, input: Res<ButtonInput<KeyCode>>) {
    use avian2d::prelude::PhysicsGizmos;

    if input.just_pressed(KeyCode::KeyP) {
        let config = store.config_mut::<PhysicsGizmos>().0;
        config.enabled = !config.enabled;
    }
}
