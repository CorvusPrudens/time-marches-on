use crate::{
    animation::AnimationSprite,
    cutscene::{
        chara::{Chara, Character},
        fragments::IntoBox,
    },
    hook::Hook,
};
use bevy::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_seedling::prelude::*;
use bevy_sequence::{combinators::delay::run_after, prelude::*};
use bevy_tween::prelude::*;
use interpolate::sprite_color;

pub fn sturgeon() -> impl IntoBox {
    (
        "how many times".chara(Chara::Sturgeon),
        "will you tell her".chara(Chara::Sturgeon),
        "how many".chara(Chara::Sturgeon),
        "will you".chara(Chara::Sturgeon),
    )
        .always()
        .once()
}

pub fn shadow_1() -> impl IntoBox {
    // TODO: for some reason we need a tuple always???
    ("you built a cage",).shadow().always().once()
}

pub fn shadow_2() -> impl IntoBox {
    ("do you need more painkillers yet?",)
        .shadow()
        .always()
        .once()
}

pub fn shadow_3() -> impl IntoBox {
    ("die already",).shadow().always().once()
}

pub fn shadow_4() -> impl IntoBox {
    ("you'll be all alone",).shadow().always().once()
}

pub fn shadow_5() -> impl IntoBox {
    ("it won't be long, now",).shadow().always().once()
}

pub fn shadow_6() -> impl IntoBox {
    ("who is she, anyway",).shadow().always().once()
}

pub fn shadow_7() -> impl IntoBox {
    ("alone",).shadow().always().once()
}

pub fn shadow_8() -> impl IntoBox {
    ("oh god, the smell",).shadow().always().once()
}

pub fn shadow_9() -> impl IntoBox {
    ("you dropped your key",).shadow().always().once()
}

pub fn final_cutscene() -> impl IntoBox {
    (
        "Pill are scattered across the floor.".narrator(),
        1.0,
        "Luna?".father(),
        "Honey, what's going on?".father(),
        2.0,
        "You gently shake her.".narrator(),
        2.0,
        "Luna, I'm scared.".father(),
        2.0,
        "She doesn't move.".narrator(),
        "She's not waking up.".distressed_narrator(),
        "She's not...".distressed_narrator(),
        2.0,
        "She's not breathing.".distressed_narrator(),
        5.0,
        "Get out of here.".distressed_narrator(),
        "Lock the door.".distressed_narrator(),
        "please...".distressed_narrator().on_start(
            |mut commands: Commands, server: Res<AssetServer>| {
                let overlay = commands
                    .spawn((
                        crate::hook::Hook,
                        HIGH_RES_LAYER,
                        AnimationSprite::repeating("textures/mega-swiggle.png", 0.1, 0..5),
                        Transform::from_xyz(0., 0., 900.)
                            .with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
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
                    ))
                    .id();

                commands.entity(overlay).animation().insert_tween_here(
                    Duration::from_secs(13),
                    EaseKind::QuadraticOut,
                    overlay
                        .into_target()
                        .with(sprite_color(Color::WHITE.with_alpha(0.0), Color::WHITE)),
                );

                let face = commands
                    .spawn((
                        Hook,
                        HIGH_RES_LAYER,
                        Transform::from_xyz(0., 0., 901.)
                            .with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
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
                    |mut writer: EventWriter<AppExit>| {
                        writer.write_default();
                    },
                    &mut commands,
                );
            },
        ),
        2.0,
        "just... forget about this".distressed_narrator(),
        "like you forget everything else".distressed_narrator(),
    )
        .always()
        .once()
}
