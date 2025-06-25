use std::marker::PhantomData;
use std::time::Duration;

use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_optix::camera::PixelSnap;
use bevy_optix::zorder::YOrigin;
use bevy_seedling::prelude::*;
use rand::Rng;

use crate::animation::{AnimationAppExt, AnimationController, AnimationSprite};
use crate::{Layer, world};

pub const PLAYER_SPEED: f32 = 70.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<PlayerContext>()
            .register_layout(
                "textures/player.png",
                TextureAtlasLayout::from_grid(UVec2::splat(48), 12, 8, None, None),
            )
            .register_required_components::<world::PlayerVessel, Player>()
            .add_systems(Update, (scaled, play_footsteps))
            .add_observer(bind)
            .add_observer(apply_movement)
            .add_observer(stop_movement)
            .add_observer(move_sprite)
            .add_observer(stop_sprite)
            .add_observer(observe_add_inhibit::<PlayerContext>)
            .add_observer(observe_remove_inhibit::<PlayerContext>);
    }
}

#[derive(Event)]
pub struct InhibitAddEvent;

#[derive(Event)]
pub struct InhibitRemoveEvent;

#[derive(Component)]
pub struct ControlInhibitor<C: InputContext> {
    inhibit_count: usize,
    context: PhantomData<fn() -> C>,
}

fn observe_add_inhibit<C: InputContext>(
    trigger: Trigger<InhibitAddEvent>,
    inhibited: Query<Has<ControlInhibitor<C>>>,
    mut commands: Commands,
) -> Result {
    if !inhibited.get(trigger.target())? {
        commands.entity(trigger.target()).remove::<Actions<C>>();
    }

    commands
        .entity(trigger.target())
        .entry::<ControlInhibitor<C>>()
        .or_insert(ControlInhibitor {
            inhibit_count: 0,
            context: PhantomData,
        })
        .and_modify(|mut c| c.inhibit_count += 1);

    Ok(())
}

fn observe_remove_inhibit<C: InputContext>(
    trigger: Trigger<InhibitRemoveEvent>,
    mut inhibited: Query<&mut ControlInhibitor<C>>,
    mut commands: Commands,
) {
    let Ok(mut inhibit) = inhibited.get_mut(trigger.target()) else {
        commands
            .entity(trigger.target())
            .insert(Actions::<C>::default());
        return;
    };

    inhibit.inhibit_count = inhibit.inhibit_count.saturating_sub(1);

    if inhibit.inhibit_count == 0 {
        commands
            .entity(trigger.target())
            .remove::<ControlInhibitor<C>>()
            .insert(Actions::<C>::default());
    }
}

pub fn remove_actions(mut commands: Commands, player: Single<Entity, With<Player>>) {
    commands.entity(*player).remove::<Actions<PlayerContext>>();
}

pub fn add_actions(mut commands: Commands, player: Single<Entity, With<Player>>) {
    commands
        .entity(*player)
        .insert(Actions::<PlayerContext>::default());
}

#[derive(Default, Component)]
#[require(
    RigidBody::Dynamic,
    LockedAxes::ROTATION_LOCKED,
    Actions<PlayerContext>,
    PixelSnap,
    YOrigin(-12.),
    Scaled(Vec2::splat(0.8)),
    FootstepTimer(Timer::new(Duration::from_millis(750), TimerMode::Repeating)),
    SpatialListener2D,
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
            .insert(AnimationSprite::repeating("textures/player.png", 0.0, [1]))
            .with_child((
                CollisionLayers::new(Layer::Player, Layer::Default),
                Collider::circle(6.0),
                Transform::from_xyz(0., -16., 0.),
                PlayerCollider,
            ));
    }
}

#[derive(Component)]
pub struct Scaled(pub Vec2);

fn scaled(mut commands: Commands, mut entities: Query<(Entity, &mut Transform, &Scaled)>) {
    for (entity, mut transform, scaled) in entities.iter_mut() {
        commands.entity(entity).remove::<Scaled>();
        transform.scale = scaled.0.extend(1.);
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

#[derive(Component)]
struct FootstepTimer(Timer);

fn play_footsteps(
    player: Single<(&mut FootstepTimer, &LinearVelocity), With<Player>>,
    mut commands: Commands,
    server: Res<AssetServer>,
    time: Res<Time>,
    mut is_moving: Local<bool>,
) {
    let (mut timer, velocity) = player.into_inner();

    if velocity.length() == 0.0 {
        timer.0.reset();
        *is_moving = false;
        return;
    }

    let just_started = !*is_moving;
    *is_moving = true;

    if timer.0.tick(time.delta()).just_finished() || just_started {
        let mut rng = rand::thread_rng();

        let sample = if rng.gen_bool(0.5) {
            "audio/sfx/step1.wav"
        } else {
            "audio/sfx/step2.wav"
        };

        commands.spawn((
            SamplePlayer::new(server.load(sample)).with_volume(Volume::Decibels(-6.0)),
            PitchRange::new(0.075),
        ));
    }
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
        [0.0, -1.0] => [0, 2],
        [0.0, 1.0] => [36, 38],
        [1.0, 0.0] => [12, 14],
        [-1.0, 0.0] => [24, 26],
        _ => unreachable!(),
    };

    if player.is_some_and(|player| player.indices.seq[0] == range[0]) {
        return;
    }

    commands
        .entity(trigger.target())
        .insert(AnimationSprite::repeating(
            "textures/player.png",
            0.4,
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
        atlas.index = animation.indices.seq[0] + 1;
    }
}
