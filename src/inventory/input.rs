use std::time::Duration;

use bevy::{
    input_focus::{InputFocus, directional_navigation::DirectionalNavigation},
    math::{CompassOctant, FloatOrd},
    picking::{
        backend::HitData,
        pointer::{Location, PointerId},
    },
    prelude::*,
    render::camera::NormalizedRenderTarget,
};
use bevy_enhanced_input::prelude::*;

use crate::PlayingState;

#[derive(InputContext)]
pub struct InventoryContext;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2, require_reset = true)]
pub struct MenuMoveAction;

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
pub struct InteractAction;

/// An unconditional unpause.
#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
pub struct UnpauseAction;

pub fn bind(
    trigger: Trigger<Binding<InventoryContext>>,
    mut actions: Query<&mut Actions<InventoryContext>>,
) -> Result {
    let mut actions = actions.get_mut(trigger.target())?;

    actions
        .bind::<UnpauseAction>()
        .to((
            KeyCode::Escape,
            KeyCode::Tab,
            KeyCode::KeyI,
            GamepadButton::Start,
        ))
        .with_conditions(JustPress::default());

    actions
        .bind::<InteractAction>()
        .to((
            KeyCode::Enter,
            KeyCode::Space,
            KeyCode::KeyE,
            KeyCode::KeyJ,
            GamepadButton::South,
        ))
        .with_conditions(JustPress::default());

    actions
        .bind::<MenuMoveAction>()
        .to((
            Cardinal::wasd_keys(),
            Cardinal::arrow_keys(),
            Cardinal::dpad_buttons(),
            Axial::left_stick().with_modifiers_each(
                DeadZone::new(DeadZoneKind::Radial).with_lower_threshold(0.15),
            ),
        ))
        .with_conditions(JustPress::default());

    Ok(())
}

pub fn navigate(
    trigger: Trigger<Fired<MenuMoveAction>>,
    mut directional_navigation: DirectionalNavigation,
) {
    let net_east_west = if trigger.value.x == 0.0 {
        0
    } else {
        trigger.value.x.signum() as i32
    };

    let net_north_south = if trigger.value.y == 0.0 {
        0
    } else {
        trigger.value.y.signum() as i32
    };

    // Compute the direction that the user is trying to navigate in
    let maybe_direction = match (net_east_west, net_north_south) {
        (0, 0) => None,
        (0, 1) => Some(CompassOctant::North),
        (1, 1) => Some(CompassOctant::NorthEast),
        (1, 0) => Some(CompassOctant::East),
        (1, -1) => Some(CompassOctant::SouthEast),
        (0, -1) => Some(CompassOctant::South),
        (-1, -1) => Some(CompassOctant::SouthWest),
        (-1, 0) => Some(CompassOctant::West),
        (-1, 1) => Some(CompassOctant::NorthWest),
        _ => None,
    };

    if let Some(direction) = maybe_direction {
        match directional_navigation.navigate(direction) {
            // In a real game, you would likely want to play a sound or show a visual effect
            // on both successful and unsuccessful navigation attempts
            Ok(entity) => {
                println!("Navigated {direction:?} successfully. {entity} is now focused.");
            }
            Err(e) => println!("Navigation failed: {e}"),
        }
    }
}

pub fn unpause(
    trigger: Trigger<Fired<UnpauseAction>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<PlayingState>>,
) -> Result {
    // remove player context, enter paused state
    commands
        .entity(trigger.target())
        .remove::<Actions<InventoryContext>>();
    next_state.set(PlayingState::Playing);

    Ok(())
}

// By sending a Pointer<Click> trigger rather than directly handling button-like interactions,
// we can unify our handling of pointer and keyboard/gamepad interactions
pub fn interact_with_focused_button(
    _: Trigger<Fired<InteractAction>>,
    input_focus: Res<InputFocus>,
    mut commands: Commands,
) {
    let Some(focused_entity) = input_focus.0 else {
        return;
    };

    commands.trigger_targets(
        Pointer::<Click> {
            target: focused_entity,
            // We're pretending that we're a mouse
            pointer_id: PointerId::Mouse,
            // This field isn't used, so we're just setting it to a placeholder value
            pointer_location: Location {
                target: NormalizedRenderTarget::Image(bevy::render::camera::ImageRenderTarget {
                    handle: Handle::default(),
                    scale_factor: FloatOrd(1.0),
                }),
                position: Vec2::ZERO,
            },
            event: Click {
                button: PointerButton::Primary,
                // This field isn't used, so we're just setting it to a placeholder value
                hit: HitData {
                    camera: Entity::PLACEHOLDER,
                    depth: 0.0,
                    position: None,
                    normal: None,
                },
                duration: Duration::from_secs_f32(0.1),
            },
        },
        focused_entity,
    );
}
