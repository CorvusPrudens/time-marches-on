use std::borrow::Cow;

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use bevy_enhanced_input::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_pretty_text::prelude::{GlyphRevealed, Reveal, TypeWriter, TypeWriterFinished};
use bevy_seedling::prelude::Volume;
use bevy_seedling::sample::{PitchRange, SamplePlayer};

use crate::GameState;
use crate::player::{Player, PlayerContext};

pub struct TextboxPlugin;

impl Plugin for TextboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<TextboxContext>()
            .add_event::<TextboxEvent>()
            .add_systems(OnEnter(GameState::Playing), spawn_textbox)
            .add_systems(Update, textbox_event)
            .add_observer(bind)
            .add_observer(close_textbox)
            .add_observer(reveal_textbox)
            .add_observer(reveal_debounce);
        //.add_systems(Update, bounds_gizmo);
    }
}

#[derive(Event)]
pub struct TextboxEvent(Vec<TextSection>);

impl TextboxEvent {
    pub fn new(sections: impl IntoIterator<Item = TextSection>) -> Self {
        let sections = sections.into_iter().collect::<Vec<_>>();
        debug_assert!(sections.len() > 0);

        Self(sections)
    }

    pub fn section(section: TextSection) -> Self {
        Self::new([section])
    }
}

#[derive(Clone)]
pub struct TextSection {
    pub text: Cow<'static, str>,
}

impl TextSection {
    pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
        Self { text: text.into() }
    }
}

fn textbox_event(
    mut commands: Commands,
    mut reader: EventReader<TextboxEvent>,
    textbox: Option<Single<&TextboxSections>>,
    player: Single<Entity, With<Player>>,
) -> Result {
    if !reader.is_empty() && textbox.is_some() || reader.len() > 1 {
        return Err("Received `TextboxEvent` while another event is being processed".into());
    }

    for event in reader.read() {
        commands.spawn(TextboxSections(event.0.iter().cloned().rev().collect()));
        commands.run_system_cached(spawn_textbox);
        commands.entity(*player).remove::<Actions<PlayerContext>>();
    }

    Ok(())
}

#[derive(Component)]
struct TextboxSections(Vec<TextSection>);

#[derive(InputContext)]
pub struct TextboxContext;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Interact;

fn bind(
    trigger: Trigger<Binding<TextboxContext>>,
    mut actions: Query<&mut Actions<TextboxContext>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();

    actions
        .bind::<Interact>()
        .to((KeyCode::KeyJ, KeyCode::Space, GamepadButton::South))
        .with_conditions(JustPress::default());
}

#[derive(Component)]
struct AwaitInput;

fn close_textbox(
    _: Trigger<Fired<Interact>>,
    mut commands: Commands,
    textbox: Single<Entity, (With<AwaitInput>, Without<RevealDeBounce>)>,
    text: Single<(Entity, &mut Text2d), With<TextboxText>>,
    sections: Single<(Entity, &mut TextboxSections)>,
    player: Single<Entity, With<Player>>,
) {
    let (entity, mut sections) = sections.into_inner();

    match sections.0.pop() {
        Some(section) => {
            let (entity, mut text) = text.into_inner();

            text.0.clear();
            text.0.extend(section.text.chars());

            commands.entity(*textbox).remove::<AwaitInput>();
            commands.entity(entity).insert(TypeWriter::cps(30.));
        }
        None => {
            commands.entity(*textbox).despawn();
            commands.entity(entity).despawn();
            commands
                .entity(*player)
                .insert(Actions::<PlayerContext>::default());
        }
    }
}

fn reveal_textbox(
    _: Trigger<Fired<Interact>>,
    mut commands: Commands,
    mut text: Single<&mut Reveal, (With<TypeWriter>, With<TextboxText>)>,
    textbox: Single<Entity, (With<Textbox>, Without<RevealDeBounce>, Without<AwaitInput>)>,
) {
    text.all();
    commands.entity(*textbox).insert(RevealDeBounce);
}

#[derive(Component)]
struct RevealDeBounce;

fn reveal_debounce(
    _: Trigger<Completed<Interact>>,
    mut commands: Commands,
    textbox: Single<Entity, With<RevealDeBounce>>,
) {
    commands.entity(*textbox).remove::<RevealDeBounce>();
}

#[derive(Component)]
#[require(Visibility, Actions<TextboxContext>)]
struct Textbox;

#[derive(Component)]
struct TextboxText;

fn spawn_textbox(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut sections: Single<&mut TextboxSections>,
) {
    let bounds = Vec2::new(
        crate::WIDTH * crate::RESOLUTION_SCALE - 80.,
        crate::HEIGHT * crate::RESOLUTION_SCALE / 3.,
    );
    let translation = Vec3::new(0., -60., 0.);

    let textbox = commands
        .spawn((
            Textbox,
            Transform::from_xyz(0., 0., 500.),
            children![(
                Sprite::from_image(server.load("textures/textbox.png"),),
                Transform::from_xyz(0., 0., -1.).with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
                HIGH_RES_LAYER,
            )],
        ))
        .id();

    let section = sections.0.pop().unwrap();
    let text = commands
        .spawn((
            TextboxText,
            TypeWriter::cps(30.),
            Text2d::new(section.text),
            TextFont {
                font: server.load("fonts/raster-forge.ttf"),
                font_size: 32.,
                ..Default::default()
            },
            TextBounds::new(bounds.x, bounds.y),
            Transform::from_translation(translation),
            Anchor::TopCenter,
            HIGH_RES_LAYER,
        ))
        .observe(glyph_reveal)
        .observe(finish)
        .id();

    commands.entity(textbox).add_child(text);
}

fn glyph_reveal(_: Trigger<GlyphRevealed>, mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        PitchRange(0.9..1.1),
        SamplePlayer {
            sample: server.load("audio/sfx/glyph.wav"),
            volume: Volume::Linear(0.5),
            ..Default::default()
        },
    ));
}

fn finish(
    _: Trigger<TypeWriterFinished>,
    mut commands: Commands,
    textbox: Single<Entity, With<Textbox>>,
) {
    commands.entity(*textbox).insert(AwaitInput);
}

//fn bounds_gizmo(
//    bounds: Query<(&GlobalTransform, &TextBounds, &Anchor, &TextLayoutInfo)>,
//    mut gizmos: Gizmos,
//) {
//    for (gt, bounds, anchor, layout) in bounds.iter() {
//        if let Some(bounds) = bounds.width.and_then(|width| {
//            bounds
//                .height
//                .and_then(|height| Some(Vec2::new(width, height)))
//        }) {
//            let bottom_left =
//                -(anchor.as_vec() + 0.5) * bounds + (bounds.y - layout.size.y) * Vec2::Y;
//            let transform = *gt * GlobalTransform::from_translation(bottom_left.extend(0.));
//
//            gizmos.rect_2d(
//                Isometry2d::from_translation(
//                    transform.translation().xy() * crate::RESOLUTION_SCALE,
//                ),
//                bounds / crate::RESOLUTION_SCALE,
//                RED,
//            );
//        }
//    }
//}
