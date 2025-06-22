use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{GameState, PlayingState, player::PlayerContext};

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
pub struct PauseAction;

pub fn bind(
    trigger: Trigger<Binding<PlayerContext>>,
    mut actions: Query<&mut Actions<PlayerContext>>,
) -> Result {
    let mut actions = actions.get_mut(trigger.target())?;

    actions
        .bind::<PauseAction>()
        .to((
            KeyCode::Escape,
            KeyCode::Tab,
            KeyCode::KeyI,
            GamepadButton::Start,
            GamepadButton::North,
        ))
        .with_conditions(JustPress::default());

    Ok(())
}

pub fn pause(
    trigger: Trigger<Fired<PauseAction>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<PlayingState>>,
) -> Result {
    // remove player context, enter paused state
    commands
        .entity(trigger.target())
        .remove::<Actions<PlayerContext>>();
    next_state.set(PlayingState::Paused);

    Ok(())
}
