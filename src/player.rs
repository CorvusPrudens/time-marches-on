use crate::GameState;
use crate::loading::TextureAssets;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

const PLAYER_SPEED: f32 = 200.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_input_context::<PlayerContext>()
            .add_observer(bind)
            .add_observer(apply_movement)
            .add_observer(stop_movement);
    }
}

#[derive(Component)]
#[require(RigidBody::Kinematic)]
pub struct Player;

#[derive(InputContext)]
pub struct PlayerContext;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct MoveAction;

fn bind(trigger: Trigger<Binding<PlayerContext>>, mut actions: Query<&mut Actions<PlayerContext>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();

    actions.bind::<MoveAction>().to((
        Cardinal::wasd_keys(),
        Cardinal::arrow_keys(),
        Cardinal::dpad_buttons(),
        Axial::left_stick()
            .with_modifiers_each(DeadZone::new(DeadZoneKind::Radial).with_lower_threshold(0.15)),
    ));
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        Sprite::from_image(textures.bevy.clone()),
        Transform::from_translation(Vec3::new(0., 0., 1.)),
        Actions::<PlayerContext>::default(),
        Player,
    ));
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
    mut velocity: Single<&mut LinearVelocity, (With<Player>, Without<BlockControls>)>,
) {
    velocity.0 = Vec2::default();
}
