use std::borrow::Cow;
use std::sync::Arc;

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use bevy_enhanced_input::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_pretty_text::prelude::{GlyphRevealed, Reveal, TypeWriter, TypeWriterFinished};
use bevy_seedling::prelude::Volume;
use bevy_seedling::sample::{PitchRange, SamplePlayer};

use crate::animation::{AnimationAppExt, AnimationSprite};
use crate::player::{Player, PlayerContext};

pub struct TextboxPlugin;

impl Plugin for TextboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<TextboxContext>()
            .register_layout(
                "textures/textbox-await.png",
                TextureAtlasLayout::from_grid(
                    UVec2::new(crate::WIDTH as u32, crate::HEIGHT as u32),
                    4,
                    1,
                    None,
                    None,
                ),
            )
            .add_event::<TextboxEvent>()
            .add_event::<TextboxClosedEvent>()
            .init_resource::<GlyphReveal>()
            .add_systems(Update, textbox_event)
            .add_observer(bind)
            .add_observer(close_textbox)
            .add_observer(reveal_textbox);
    }
}

pub fn glyph_sample(glyph: &'static str) -> String {
    format!("audio/sfx/glyph/{}", glyph)
}

/// Spawn a textbox and present each `TextBlurb` in sequence with breaks.
#[derive(Event)]
pub struct TextboxEvent(Vec<TextBlurb>);

#[allow(unused)]
impl TextboxEvent {
    pub fn new(sections: impl IntoIterator<Item = TextBlurb>) -> Self {
        let sections = sections.into_iter().collect::<Vec<_>>();
        debug_assert!(!sections.is_empty());

        Self(sections)
    }

    pub fn section(section: TextBlurb) -> Self {
        Self::new([section])
    }
}

#[derive(Clone)]
pub struct TextBlurb {
    text: Cow<'static, str>,
    character: Option<CharacterSprite>,
    glyph: Arc<dyn Fn(&mut Commands, &AssetServer) + Send + Sync>,
}

impl TextBlurb {
    /// New text blurb with relative asset path:
    ///   - character: `textures/characters/`
    pub fn new(
        text: impl Into<Cow<'static, str>>,
        character: Option<&str>,
        glyph: impl Fn(&mut Commands, &AssetServer) + Send + Sync + 'static,
    ) -> Self {
        Self {
            text: text.into(),
            character: character.map(CharacterSprite::new),
            glyph: Arc::new(glyph),
        }
    }

    pub fn narrator(text: impl Into<Cow<'static, str>>) -> Self {
        // lol
        Self::new(text, None, |commands, server| {
            commands.spawn((
                PitchRange::new(0.02),
                SamplePlayer {
                    sample: server.load(glyph_sample("low.wav")),
                    volume: Volume::Linear(0.5),
                    ..Default::default()
                },
            ));
        })
    }

    pub fn main_character(text: impl Into<Cow<'static, str>>) -> Self {
        Self::new(text, Some("main.png"), |commands, server| {
            commands.spawn((
                PitchRange::new(0.05),
                SamplePlayer {
                    sample: server.load(glyph_sample("medium.wav")),
                    volume: Volume::Linear(0.5),
                    ..Default::default()
                },
            ));
        })
    }
}

#[derive(Clone)]
pub struct CharacterSprite(String);

impl CharacterSprite {
    /// New character sprite with relative asset path:
    ///   - character: `textures/characters/`
    pub fn new(character: impl AsRef<str>) -> Self {
        Self(format!("textures/characters/{}", character.as_ref()))
    }
}

#[derive(Component)]
pub struct CharacterSpriteEntity;

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
        commands.entity(*player).remove::<Actions<PlayerContext>>();
        commands.run_system_cached(spawn_textbox);
        commands.run_system_cached(pop_next_section);
    }

    Ok(())
}

#[derive(Component)]
struct TextboxSections(Vec<TextBlurb>);

#[derive(InputContext)]
pub struct TextboxContext;

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
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

/// An event emitted when a textbox is closed by the user.
#[derive(Debug, Event)]
pub struct TextboxClosedEvent;

#[derive(Component)]
struct AwaitInput;

fn close_textbox(
    _: Trigger<Fired<Interact>>,
    mut commands: Commands,
    textbox: Single<Entity, With<AwaitInput>>,
    sections: Single<(Entity, &TextboxSections)>,
    player: Single<Entity, With<Player>>,
    mut writer: EventWriter<TextboxClosedEvent>,
) {
    let (entity, sections) = sections.into_inner();
    match sections.0.is_empty() {
        false => {
            commands.run_system_cached(pop_next_section);
        }
        true => {
            commands.entity(*textbox).despawn();
            commands.entity(entity).despawn();
            commands
                .entity(*player)
                .insert(Actions::<PlayerContext>::default());

            writer.write(TextboxClosedEvent);
        }
    }
}

fn reveal_textbox(
    _: Trigger<Fired<Interact>>,
    mut text: Single<&mut Reveal, (With<TypeWriter>, With<TextboxText>)>,
) {
    // don't insert `AwaitInput` so that `close_textbox` does not also run,
    // wait for `TypeWriterFinished` to fire!
    text.all();
}

#[derive(Component)]
#[require(Visibility, Actions<TextboxContext>)]
struct Textbox;

#[derive(Component)]
struct TextboxText;

pub fn spawn_textbox(server: Res<AssetServer>, mut commands: Commands) {
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
                Sprite::from_image(server.load("textures/textbox.png")),
                Transform::from_xyz(0., 0., -2.).with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
                HIGH_RES_LAYER,
            )],
        ))
        .observe(await_input_visual)
        .observe(remove_await_input_visual)
        .id();

    let text = commands
        .spawn((
            TextboxText,
            Text2d::default(),
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

fn pop_next_section(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut sections: Single<&mut TextboxSections>,
    mut reveal: ResMut<GlyphReveal>,
    text: Single<(Entity, &mut Text2d), With<TextboxText>>,
    textbox: Single<Entity, With<Textbox>>,
    old_character: Option<Single<Entity, With<CharacterSpriteEntity>>>,
) {
    let section = sections.0.pop().unwrap();
    reveal.0 = Some(section.glyph.clone());

    if let Some(old_character) = old_character {
        commands.entity(*old_character).despawn();
    }

    let (text_entity, mut text) = text.into_inner();
    text.0.clear();
    text.0.extend(section.text.chars());
    commands.entity(text_entity).insert(TypeWriter::cps(30.));

    let mut textbox = commands.entity(*textbox);
    textbox.remove::<AwaitInput>();

    if let Some(character) = &section.character {
        textbox.with_child((
            CharacterSpriteEntity,
            Sprite::from_image(server.load(&character.0)),
            Transform::from_xyz(0., 0., -3.).with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
            HIGH_RES_LAYER,
        ));
    }
}

#[derive(Component)]
struct AwaitinputVisual;

fn await_input_visual(trigger: Trigger<OnAdd, AwaitInput>, mut commands: Commands) {
    commands.entity(trigger.target()).with_child((
        AnimationSprite::repeating("textures/textbox-await.png", 0.2, 0..4),
        Transform::from_xyz(0., 0., -1.).with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
        HIGH_RES_LAYER,
    ));
}

fn remove_await_input_visual(
    _: Trigger<OnRemove, AwaitInput>,
    mut commands: Commands,
    visual: Single<Entity, With<AwaitinputVisual>>,
) {
    commands.entity(*visual).despawn();
}

#[derive(Default, Resource)]
struct GlyphReveal(Option<Arc<dyn Fn(&mut Commands, &AssetServer) + Send + Sync>>);

fn glyph_reveal(
    _: Trigger<GlyphRevealed>,
    mut commands: Commands,
    server: Res<AssetServer>,
    reveal: Res<GlyphReveal>,
) {
    if let Some(reveal) = &reveal.0 {
        reveal(&mut commands, &server);
    }
}

fn finish(
    _: Trigger<TypeWriterFinished>,
    mut commands: Commands,
    textbox: Single<Entity, With<Textbox>>,
) {
    commands.entity(*textbox).insert(AwaitInput);
}
