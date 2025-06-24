use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::player::{PlayerCollider, PlayerContext};
use crate::textbox::{TextBlurb, TextboxEvent};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, remove_interacted)
            .add_systems(Update, interact_collider)
            .add_observer(interact)
            .add_observer(interactable)
            .add_observer(bind);
    }
}

/// Checks for interactions from the player.
///
/// Inserts `Interacted` for one frame.
#[derive(Default, Component)]
#[require(Sensor, CollidingEntities)]
pub struct Interactable;

/// Marks an entity as being in an active interaction.
///
/// Lasts one frame.
#[derive(Component)]
pub struct Interacted;

fn interactable(
    _: Trigger<Fired<InteractAction>>,
    mut commands: Commands,
    player: Single<(Entity, &Transform), With<PlayerCollider>>,
    interactables: Query<(Entity, &GlobalTransform, &CollidingEntities), With<Interactable>>,
) {
    let (player, transform) = player.into_inner();

    let mut interactions = interactables
        .iter()
        .filter(|(_, _, colliding)| colliding.contains(&player))
        .collect::<Vec<_>>();
    interactions.sort_unstable_by(|(_, gt1, _), (_, gt2, _)| {
        gt1.translation()
            .xy()
            .distance_squared(transform.translation.xy())
            .total_cmp(
                &gt2.translation()
                    .xy()
                    .distance_squared(transform.translation.xy()),
            )
    });

    if let Some((entity, _, _)) = interactions.first() {
        commands.entity(*entity).insert(Interacted);
    }
}

fn remove_interacted(mut commands: Commands, entities: Query<Entity, With<Interacted>>) {
    for entity in entities.iter() {
        commands.entity(entity).remove::<Interacted>();
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
pub struct InteractAction;

fn bind(
    trigger: Trigger<Binding<PlayerContext>>,
    mut actions: Query<&mut Actions<PlayerContext>>,
) -> Result {
    let mut actions = actions.get_mut(trigger.target())?;

    actions
        .bind::<InteractAction>()
        .to((
            KeyCode::KeyJ,
            KeyCode::KeyE,
            KeyCode::Space,
            GamepadButton::South,
        ))
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

    writer.write(TextboxEvent::section(TextBlurb::main_character(
        interactor.flavor.clone(),
    )));

    Ok(())
}
