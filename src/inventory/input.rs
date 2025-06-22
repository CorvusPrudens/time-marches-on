use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{GameState, PlayingState};

#[derive(InputContext)]
pub struct InventoryContext;

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

    Ok(())
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
