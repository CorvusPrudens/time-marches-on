use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_optix::camera::PixelSnap;

use crate::world;

const PLAYER_SPEED: f32 = 100.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<PlayerContext>()
            .register_required_components::<world::PlayerVessel, Player>()
            .add_observer(bind)
            .add_observer(apply_movement)
            .add_observer(stop_movement);
    }
}

#[derive(Default, Component)]
#[require(
    RigidBody::Dynamic,
    LockedAxes::ROTATION_LOCKED,
    Actions<PlayerContext>,
    PixelSnap,
    Collider::rectangle(8.0, 16.0)
)]
#[component(on_insert = Self::bind_camera)]
pub struct Player;

impl Player {
    fn bind_camera(mut world: DeferredWorld, _: HookContext) {
        world
            .commands()
            .run_system_cached(bevy_optix::camera::bind_camera::<Player>);
    }
}

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
    if velocity.0.x != 0.0 && velocity.0.x.abs() < f32::EPSILON {
        velocity.0.x = 0.;
    }
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
