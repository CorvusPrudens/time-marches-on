use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use bevy_enhanced_input::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_pretty_text::prelude::{GlyphRevealed, Reveal, TypeWriter, TypeWriterFinished};
use bevy_seedling::prelude::Volume;
use bevy_seedling::sample::{PitchRange, SamplePlayer};

use crate::GameState;

pub struct TextboxPlugin;

impl Plugin for TextboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<TextboxContext>()
            .add_systems(OnEnter(GameState::Playing), spawn_textbox)
            .add_observer(bind)
            .add_observer(close_textbox)
            .add_observer(reveal_textbox)
            .add_observer(reveal_debounce);
        //.add_systems(Update, bounds_gizmo);
    }
}

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
        .to((KeyCode::Space, KeyCode::Enter));
}

#[derive(Component)]
struct AwaitInput;

fn close_textbox(
    _: Trigger<Fired<Interact>>,
    mut commands: Commands,
    textbox: Single<Entity, (With<AwaitInput>, Without<RevealDeBounce>)>,
) {
    commands.entity(*textbox).despawn();
}

fn reveal_textbox(
    _: Trigger<Fired<Interact>>,
    mut commands: Commands,
    mut text: Single<&mut Reveal, With<TypeWriter>>,
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

fn spawn_textbox(mut commands: Commands, server: Res<AssetServer>) {
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

    let text = commands
        .spawn((
            TypeWriter::cps(30.),
            Text2d::new("My, my... I think that I am forgetting something..."),
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
