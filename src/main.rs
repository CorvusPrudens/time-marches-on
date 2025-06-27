#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]

use avian2d::prelude::{Gravity, PhysicsLayer};
use bevy::DefaultPlugins;
use bevy::app::{App, FixedMainScheduleOrder};
use bevy::asset::AssetMetaCheck;
use bevy::ecs::query::QueryFilter;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy::winit::WinitWindows;
use bevy_optix::pixel_perfect::CanvasDimensions;
use std::io::Cursor;
use winit::window::Icon;

mod animation;
mod audio;
mod callback;
mod cutscene;
mod cutscenes;
mod entities;
mod fragments;
mod hook;
mod interactions;
mod inventory;
mod levels;
mod loading;
mod menu;
mod notes;
mod player;
mod textbox;
#[allow(unused)]
mod world;

pub const WIDTH: f32 = 256.;
pub const HEIGHT: f32 = 144.;
pub const RESOLUTION_SCALE: f32 = 5.;

pub const TILE_SIZE: f32 = 16.;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // TODO: Rename
                    title: "Time Marches On".to_string(),
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
            .set(ImagePlugin::default_nearest())
            .set(LogPlugin {
                level: Level::INFO,
                ..Default::default()
            }),
        bevy_tween::DefaultTweenPlugins,
        bevy_enhanced_input::EnhancedInputPlugin,
        avian2d::PhysicsPlugins::new(Avian).with_length_unit(8.),
        bevy_optix::pixel_perfect::PixelPerfectPlugin(CanvasDimensions {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            pixel_scale: RESOLUTION_SCALE,
        }),
        bevy_optix::debug::DebugPlugin,
        bevy_optix::camera::CameraAnimationPlugin,
        bevy_optix::zorder::ZOrderPlugin,
        bevy_pretty_text::PrettyTextPlugin,
        bevy_ldtk_scene::LdtkScenePlugin,
        world::TimeMarchesOnPlugin,
        bevy_sequence::SequencePlugin,
        bevy_light_2d::plugin::Light2dPlugin,
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
        inventory::InventoryPlugin,
        levels::LevelPlugin,
        entities::EntityPlugin,
        animation::AnimationPlugin,
        hook::HookPlugin,
        cutscene::CutscenePlugin,
        cutscenes::CutscenePlugin,
        notes::NotesPlugin,
        audio::AudioPlugin,
    ))
    .init_state::<GameState>()
    .add_sub_state::<PlayingState>()
    .init_schedule(Avian)
    .insert_resource(Gravity(Vec2::ZERO))
    .add_systems(Startup, set_window_icon);

    #[cfg(not(feature = "web-audio"))]
    app.add_plugins(bevy_seedling::SeedlingPlugin::default());

    #[cfg(feature = "web-audio")]
    app.add_plugins(
        bevy_seedling::SeedlingPlugin::<firewheel_web_audio::WebAudioBackend> {
            config: Default::default(),
            stream_config: Default::default(),
            spawn_default_pool: true,
            pool_size: 4..=32,
        },
    );

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
    Hook,
    Playing,
}

#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash)]
#[source(GameState = GameState::Playing)]
enum PlayingState {
    #[default]
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Avian;

#[derive(Default, Clone, Copy, PartialEq, Eq, PhysicsLayer)]
pub enum Layer {
    #[default]
    Default,
    Player,
    Other,
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

pub fn despawn_entities<F: QueryFilter>(mut commands: Commands, entities: Query<Entity, F>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
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
fn enable_avian_debug(
    mut store: ResMut<GizmoConfigStore>,
    input: Res<ButtonInput<KeyCode>>,
    mut setup: Local<bool>,
) {
    use avian2d::prelude::PhysicsGizmos;

    if !*setup {
        let config = store.config_mut::<PhysicsGizmos>().0;
        config.enabled = false;
        *setup = true;
    }

    if input.just_pressed(KeyCode::KeyP) {
        let config = store.config_mut::<PhysicsGizmos>().0;
        config.enabled = !config.enabled;
    }
}
