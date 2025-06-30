use super::DoorDisabled;
use crate::sequence::{ObserverSequence, con, delay};
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
use bevy_sequence::prelude::*;

pub struct TeaPlugin;

impl Plugin for TeaPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<world::TeaSpawner, TeaSpawner>()
            .register_required_components::<world::TeaTable, TeaTable>()
            .register_required_components_with::<world::Table, _>(|| YOrigin(-8.));
    }
}

#[derive(Default, Component)]
#[require(
    Sensor,
    CollisionEventsEnabled,
    CollisionLayers::new(crate::Layer::Default, crate::Layer::Player),
    Collider::rectangle(96.0, 64.0)
)]
pub struct TeaSpawner;

#[derive(Default, Component)]
#[require(Sensor, Interactable, Collider::rectangle(64.0, 64.0))]
pub struct TeaTable;

fn spawn_luna(
    luna: Query<&GlobalTransform, With<world::LunaTea>>,
    mut commands: Commands,
) -> Result {
    let luna_transform = luna.single()?;

    commands.spawn((
        AnimationSprite::repeating("textures/luna.png", 0.0, [50]),
        (*luna_transform * GlobalTransform::from_xyz(8., -8., 0.)).compute_transform(),
        YOrigin(-14.),
    ));

    Ok(())
}

#[derive(Event)]
struct CutsceneDone;

pub fn tea_sequence() -> impl IntoFragment<ObserverSequence> {
    (
        con(
            |trigger: Trigger<OnCollisionStart>, tea_spawner: Single<Entity, With<TeaSpawner>>| {
                trigger.target() == *tea_spawner
            },
        )
        .on_end(spawn_luna),
        con(
            |trigger: Trigger<OnAdd, Interacted>,
             table: Query<Entity, With<world::TeaTable>>,
             mut commands: Commands|
             -> Result<bool> {
                let table = table.single()?;
                if trigger.target() != table {
                    return Ok(false);
                }

                // spawn the cutscene
                crate::cutscenes::tea::tea_cutscene()
                    .on_end(|mut commands: Commands| commands.trigger(CutsceneDone))
                    .spawn_box(&mut commands);

                Ok(true)
            },
        ),
        con(|_: Trigger<CutsceneDone>| ()),
        delay(5.0).on_end(
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
                    commands.entity(entity).remove::<DoorDisabled>().insert((
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
        ),
    )
        .always()
        .once()
}
