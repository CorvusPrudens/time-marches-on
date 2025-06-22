use std::time::Duration;

use bevy::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_seedling::prelude::{RepeatMode, Volume};
use bevy_seedling::sample::SamplePlayer;
use bevy_sequence::combinators::delay::run_after;
use bevy_tween::interpolate::sprite_color;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind};
use bevy_tween::tween::IntoTarget;

use crate::GameState;
use crate::animation::{AnimationAppExt, AnimationSprite};

pub struct HookPlugin;

impl Plugin for HookPlugin {
    fn build(&self, app: &mut App) {
        app.register_layout(
            "textures/mega-swiggle.png",
            TextureAtlasLayout::from_grid(
                UVec2::new(crate::WIDTH as u32, crate::HEIGHT as u32),
                5,
                1,
                None,
                None,
            ),
        )
        .add_systems(OnEnter(GameState::Hook), spawn);
    }
}

#[derive(Component)]
struct Hook;

fn spawn(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        Hook,
        HIGH_RES_LAYER,
        AnimationSprite::repeating("textures/mega-swiggle.png", 0.1, 0..5),
        Transform::from_xyz(0., 0., 900.).with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
        children![
            SamplePlayer {
                sample: server.load("audio/sfx/whispers.wav"),
                volume: Volume::Linear(0.5),
                repeat_mode: RepeatMode::RepeatEndlessly,
            },
            SamplePlayer {
                sample: server.load("audio/sfx/hook.wav"),
                volume: Volume::Linear(0.5),
                ..Default::default()
            },
            SamplePlayer {
                sample: server.load("audio/sfx/wake-up.wav"),
                volume: Volume::Linear(0.5),
                ..Default::default()
            },
            SamplePlayer {
                sample: server.load("audio/sfx/many-whispers.wav"),
                volume: Volume::Linear(0.5),
                ..Default::default()
            },
        ],
    ));

    let face = commands
        .spawn((
            Hook,
            HIGH_RES_LAYER,
            Transform::from_xyz(0., 0., 901.).with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
            Sprite::from_image(server.load("textures/face.png")),
        ))
        .id();
    commands.entity(face).animation().insert_tween_here(
        Duration::from_secs(13),
        EaseKind::QuadraticOut,
        face.into_target()
            .with(sprite_color(Color::WHITE.with_alpha(0.0), Color::WHITE)),
    );

    run_after(
        Duration::from_secs(9),
        |mut commands: Commands, entities: Query<Entity, With<Hook>>| {
            commands.set_state(GameState::Playing);
            for entity in entities.iter() {
                commands.entity(entity).despawn();
            }
        },
        &mut commands,
    );
}
