use avian2d::prelude::{Collider, Collisions, Sensor};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::player::PlayerContext;
use crate::textbox::{TextSection, TextboxEvent};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, interact_collider)
            .add_observer(interact)
            .add_observer(bind);
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct InteractAction;

fn bind(
    trigger: Trigger<Binding<PlayerContext>>,
    mut actions: Query<&mut Actions<PlayerContext>>,
) -> Result {
    let mut actions = actions.get_mut(trigger.target())?;

    actions
        .bind::<InteractAction>()
        .to((KeyCode::KeyJ, KeyCode::Space, GamepadButton::South))
        .with_conditions(JustPress::default());

    Ok(())
}

fn interact_collider(
    q: Query<(Entity, &crate::world::Interaction), Added<crate::world::Interaction>>,
    mut commands: Commands,
) {
    for (entity, interaction) in &q {
        commands.entity(entity).insert((
            Collider::rectangle(interaction.width, interaction.height),
            Sensor,
        ));
    }
}

fn interact(
    trigger: Trigger<Fired<InteractAction>>,
    interactor: Query<&GlobalTransform>,
    interactions: Query<(&crate::world::Interaction, &GlobalTransform)>,
    collisions: Collisions,
    mut writer: EventWriter<TextboxEvent>,
) -> Result {
    let target = trigger.target();
    let interactor = interactor.get(target)?;
    let interactor_translation = interactor.translation().xy();

    let mut closest_distance = f32::MAX;
    let mut closest_interactor = None;
    for pair in collisions.collisions_with(trigger.target()) {
        let other = if target == pair.collider1 {
            pair.collider2
        } else {
            pair.collider1
        };

        let Ok((interactor, transform)) = interactions.get(other) else {
            continue;
        };

        let interact_transform = transform.translation().xy();

        let distance = interactor_translation.distance_squared(interact_transform);

        if distance < closest_distance {
            closest_distance = distance;
            closest_interactor = Some(interactor);
        }
    }

    let Some(interactor) = closest_interactor else {
        return Ok(());
    };

    writer.write(TextboxEvent::section(TextSection::new(
        interactor.flavor.clone(),
    )));

    Ok(())
}
