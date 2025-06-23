use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_optix::camera::PixelSnap;
use bevy_optix::zorder::YOrigin;

use crate::animation::{AnimationAppExt, AnimationController, AnimationSprite};
use crate::{Layer, world};

const PLAYER_SPEED: f32 = 70.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<PlayerContext>()
            .register_layout(
                "textures/main-character.png",
                TextureAtlasLayout::from_grid(UVec2::splat(32), 12, 1, None, None),
            )
            .register_required_components::<world::PlayerVessel, Player>()
            .add_observer(bind)
            .add_observer(apply_movement)
            .add_observer(stop_movement)
            .add_observer(move_sprite)
            .add_observer(stop_sprite);
    }
}

#[derive(Default, Component)]
#[require(
    RigidBody::Dynamic,
    LockedAxes::ROTATION_LOCKED,
    Actions<PlayerContext>,
    PixelSnap,
    YOrigin(-8.),
)]
#[component(on_insert = Self::bind_camera)]
pub struct Player;

impl Player {
    fn bind_camera(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .run_system_cached(bevy_optix::camera::bind_camera::<Player>);
        world
            .commands()
            .entity(ctx.entity)
            .insert(AnimationSprite::repeating(
                "textures/main-character.png",
                0.5,
                0..1,
            ))
            .with_child((
                CollisionLayers::new(Layer::Player, Layer::Default),
                Collider::rectangle(8.0, 8.0),
                Transform::from_xyz(0., -6., 0.),
                PlayerCollider,
            ));
    }
}

#[derive(Component)]
pub struct PlayerCollider;

#[derive(InputContext)]
pub struct PlayerContext;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct MoveAction;

fn bind(
    trigger: Trigger<Binding<PlayerContext>>,
    mut actions: Query<&mut Actions<PlayerContext>>,
) -> Result {
    let mut actions = actions.get_mut(trigger.target())?;

    actions.bind::<MoveAction>().to((
        Cardinal::wasd_keys(),
        Cardinal::arrow_keys(),
        Cardinal::dpad_buttons(),
        Axial::left_stick()
            .with_modifiers_each(DeadZone::new(DeadZoneKind::Radial).with_lower_threshold(0.15)),
    ));

    Ok(())
}

#[derive(Default, Component)]
pub struct BlockControls;

fn apply_movement(
    trigger: Trigger<Fired<MoveAction>>,
    mut velocity: Single<&mut LinearVelocity, (With<Player>, Without<BlockControls>)>,
) {
    velocity.0 = trigger.value.clamp_length(0., 1.) * PLAYER_SPEED;
}

fn move_sprite(
    trigger: Trigger<Fired<MoveAction>>,
    mut commands: Commands,
    mut last: Local<Vec2>,
    player: Single<Option<&AnimationController>, With<Player>>,
) {
    let v = trigger.value;
    if *last == v && player.is_some() {
        return;
    }
    *last = v;

    let dir = if v.x.abs() < v.y.abs() {
        Vec2::new(0.0, if v.y.is_sign_positive() { 1. } else { -1. })
    } else {
        Vec2::new(if v.x.is_sign_positive() { 1. } else { -1. }, 0.0)
    };

    let range = match dir.to_array() {
        [0.0, -1.0] => 1..3,
        [0.0, 1.0] => 4..6,
        [1.0, 0.0] => 7..9,
        [-1.0, 0.0] => 10..12,
        _ => unreachable!(),
    };

    if player.is_some_and(|player| player.indices.seq[0] == range.start) {
        return;
    }

    commands
        .entity(trigger.target())
        .insert(AnimationSprite::repeating(
            "textures/main-character.png",
            0.2,
            range,
        ));
}

fn stop_movement(
    _: Trigger<Completed<MoveAction>>,
    player: Single<(&mut LinearVelocity, &mut Transform), (With<Player>, Without<BlockControls>)>,
) {
    let (mut velocity, mut transform) = player.into_inner();
    velocity.0 = Vec2::default();
    transform.translation = transform
        .translation
        .xy()
        .round()
        .extend(transform.translation.z)
}

fn stop_sprite(
    _: Trigger<Completed<MoveAction>>,
    mut commands: Commands,
    player: Single<(Entity, &mut Sprite, &AnimationController), With<Player>>,
) {
    let (entity, mut sprite, animation) = player.into_inner();
    commands.entity(entity).remove::<AnimationController>();
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = animation.indices.seq[0] - 1;
    }
}
