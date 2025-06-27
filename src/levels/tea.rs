use std::time::Duration;

use crate::{
    animation::AnimationSprite,
    cutscene::fragments::IntoBox,
    interactions::{Interactable, Interacted},
    world,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_optix::zorder::YOrigin;
use bevy_seedling::prelude::*;
use bevy_sequence::{combinators::delay::run_after, prelude::FragmentExt};

pub struct TeaPlugin;

impl Plugin for TeaPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<world::TeaSpawner, TeaSpawner>()
            .register_required_components::<world::TeaTable, TeaTable>()
            .register_required_components_with::<world::Table, _>(|| YOrigin(-8.))
            .init_state::<TeaState>()
            .add_systems(OnEnter(TeaState::TriggerReady), ready_trigger)
            .add_systems(OnEnter(TeaState::SpawnLuna), spawn_tea)
            .add_observer(tea_trigger);
    }
}

#[derive(Default, Component)]
#[require(
    Sensor,
    ColliderDisabled,
    CollisionEventsEnabled,
    CollisionLayers::new(crate::Layer::Default, crate::Layer::Player),
    Collider::rectangle(96.0, 64.0)
)]
pub struct TeaSpawner;

#[derive(Default, Component)]
#[require(
    Sensor,
    ColliderDisabled,
    Interactable,
    Collider::rectangle(64.0, 64.0)
)]
pub struct TeaTable;

#[derive(Default, States, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TeaState {
    #[default]
    Inactive,
    TriggerReady,
    SpawnLuna,
    Complete,
}

fn ready_trigger(spawner: Query<Entity, With<TeaSpawner>>, mut commands: Commands) {
    for spawner in spawner.iter() {
        commands.entity(spawner).remove::<ColliderDisabled>();
    }
}

fn tea_trigger(
    trigger: Trigger<OnCollisionStart>,
    tea: Single<(Entity, &TeaSpawner)>,
    mut commands: Commands,
    mut next: ResMut<NextState<TeaState>>,
) -> Result {
    let (tea, _) = tea.into_inner();
    if trigger.target() != tea {
        return Ok(());
    }

    commands.entity(tea).despawn();
    next.set(TeaState::SpawnLuna);

    Ok(())
}

fn spawn_tea(
    luna: Query<(&world::LunaTea, &GlobalTransform)>,
    table: Query<Entity, With<world::TeaTable>>,
    mut commands: Commands,
) -> Result {
    let (_, luna_transform) = luna.single()?;
    let table = table.single()?;

    commands.spawn((
        AnimationSprite::repeating("textures/luna.png", 0.0, [50]),
        (*luna_transform * GlobalTransform::from_xyz(8., -8., 0.)).compute_transform(),
        YOrigin(-14.),
    ));

    commands
        .entity(table)
        .remove::<ColliderDisabled>()
        .observe(watch_table);

    Ok(())
}

fn watch_table(trigger: Trigger<OnAdd, Interacted>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .remove::<CollidingEntities>();

    crate::cutscenes::tea::tea_cutscene()
        .on_end(
            |mut state: ResMut<NextState<TeaState>>, mut commands: Commands| {
                state.set(TeaState::Complete);

                run_after(
                    Duration::from_secs(5),
                    |mut commands: Commands,
                     server: Res<AssetServer>,
                     cracked_door: Query<(Entity, &world::CrackedSideDoor1)>,
                     side_door: Query<(Entity, &world::SideDoor1)>| {
                        for (entity, _) in side_door
                            .iter()
                            .filter(|(_, door)| door.id as usize == 8392)
                        {
                            commands.entity(entity).despawn();
                        }
                        for (entity, _) in cracked_door
                            .iter()
                            .filter(|(_, door)| door.id as usize == 8392)
                        {
                            commands
                                .entity(entity)
                                .remove::<ColliderDisabled>()
                                .insert((
                                    Visibility::Visible,
                                    // spatializing sound on door
                                    PlaybackSettings {
                                        on_complete: OnComplete::Remove,
                                        ..Default::default()
                                    },
                                    SamplePlayer {
                                        sample: server.load("audio/sfx/door-open.wav"),
                                        //volume: Volume::Linear(1.25),
                                        ..Default::default()
                                    },
                                    crate::audio::SpatialPool,
                                ));
                        }
                    },
                    &mut commands,
                );
            },
        )
        .spawn_box(&mut commands);
}
