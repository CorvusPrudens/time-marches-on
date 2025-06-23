use std::time::Duration;

use bevy::prelude::*;
use bevy_enhanced_input::events::Fired;
use bevy_enhanced_input::prelude::Actions;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_sequence::combinators::delay::run_after;
use bevy_tween::interpolate::{sprite_color, translation};
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind};
use bevy_tween::tween::IntoTarget;

use crate::player::{self, Player, PlayerContext};
use crate::textbox::{Interact, TextboxContext};

pub struct NotesPlugin;

impl Plugin for NotesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NoteEvent>().add_systems(Update, note_event);
    }
}

#[derive(Clone, Copy, Event)]
pub struct NoteEvent(pub &'static str);

const FADE_DUR: f32 = 0.2;
const SLIDE_DUR: f32 = 1.;

#[derive(Component)]
struct Note;

#[derive(Component)]
struct Fade;

#[derive(Component)]
struct TheNote;

fn note_event(
    mut commands: Commands,
    mut reader: EventReader<NoteEvent>,
    player: Single<Entity, With<Player>>,
    server: Res<AssetServer>,
) {
    if reader.is_empty() {
        return;
    }

    let note = *reader.read().next().unwrap();
    debug_assert!(reader.is_empty());
    reader.clear();

    commands.entity(*player).remove::<Actions<PlayerContext>>();

    let entity = commands.spawn_empty().id();
    commands
        .entity(entity)
        .insert((
            Note,
            Fade,
            HIGH_RES_LAYER,
            Sprite::from_color(Color::NONE, Vec2::new(crate::WIDTH, crate::HEIGHT)),
            Transform::from_translation(Vec3::new(0.0, 0.0, 900.0))
                .with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
        ))
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(FADE_DUR),
            EaseKind::Linear,
            entity
                .into_target()
                .with(sprite_color(Color::NONE, Color::BLACK.with_alpha(0.9))),
        );

    let entity = commands.spawn_empty().id();
    commands
        .entity(entity)
        .insert((
            Note,
            TheNote,
            HIGH_RES_LAYER,
            Sprite::from_image(server.load(format!("textures/notes/{}", note.0))),
            Transform::from_xyz(0., 0., 901.0).with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
        ))
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(SLIDE_DUR),
            EaseKind::QuarticOut,
            entity.into_target().with(translation(
                Vec3::new(0., -crate::HEIGHT * crate::RESOLUTION_SCALE, 901.0),
                Vec3::new(0., 0., 901.0),
            )),
        );

    commands
        .spawn((Actions::<TextboxContext>::default(), Note))
        .observe(exit);
}

fn exit(
    _: Trigger<Fired<Interact>>,
    mut commands: Commands,
    fade: Single<Entity, With<Fade>>,
    note: Single<Entity, With<TheNote>>,
) {
    commands.entity(*fade).animation().insert_tween_here(
        Duration::from_secs_f32(FADE_DUR),
        EaseKind::Linear,
        fade.into_target()
            .with(sprite_color(Color::BLACK.with_alpha(0.9), Color::NONE)),
    );

    commands.entity(*note).animation().insert_tween_here(
        Duration::from_secs_f32(SLIDE_DUR),
        EaseKind::QuadraticIn,
        note.into_target().with(translation(
            Vec3::new(0., 0., 901.0),
            Vec3::new(0., -crate::HEIGHT * crate::RESOLUTION_SCALE, 901.0),
        )),
    );

    run_after(
        Duration::from_secs_f32(SLIDE_DUR.max(FADE_DUR)),
        crate::despawn_entities::<With<Note>>,
        &mut commands,
    );
    run_after(
        Duration::from_secs_f32(SLIDE_DUR.max(FADE_DUR)),
        player::add_actions,
        &mut commands,
    );
}
