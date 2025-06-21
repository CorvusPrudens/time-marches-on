#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]

use avian2d::prelude::{Gravity, PhysicsLayer};
use bevy::DefaultPlugins;
use bevy::app::{App, FixedMainScheduleOrder};
use bevy::asset::AssetMetaCheck;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy::winit::WinitWindows;
use bevy_optix::pixel_perfect::CanvasDimensions;
use std::io::Cursor;
use winit::window::Icon;

use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

mod loading;
mod menu;
mod player;

pub const WIDTH: f32 = 640.;
pub const HEIGHT: f32 = 360.;
pub const RESOLUTION_SCALE: f32 = 2.;

fn main() {
    let mut app = App::new();

    #[cfg(debug_assertions)]
    app.add_systems(Update, close_on_escape);

    app.insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // TODO: Rename
                        title: "Bevy game".to_string(),
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
                }),
            bevy_tween::DefaultTweenPlugins,
            //bevy_seedling::SeedlingPlugin {
            //    ..Default::default()
            //},
            bevy_enhanced_input::EnhancedInputPlugin,
            //avian2d::debug_render::PhysicsDebugPlugin::new(Avian),
            avian2d::PhysicsPlugins::new(Avian).with_length_unit(8.),
            bevy_optix::pixel_perfect::PixelPerfectPlugin(CanvasDimensions {
                width: WIDTH as u32,
                height: HEIGHT as u32,
                pixel_scale: RESOLUTION_SCALE,
            }),
            bevy_optix::debug::DebugPlugin,
            bevy_pretty_text::PrettyTextPlugin,
        ))
        .add_plugins((LoadingPlugin, MenuPlugin, PlayerPlugin))
        .init_state::<GameState>()
        .init_schedule(Avian)
        .insert_resource(Gravity(Vec2::ZERO))
        .add_systems(Startup, set_window_icon);

    app.world_mut()
        .resource_mut::<FixedMainScheduleOrder>()
        .insert_after(FixedPostUpdate, Avian);

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
