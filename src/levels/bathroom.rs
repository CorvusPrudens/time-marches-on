use std::time::Duration;

use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_ldtk_scene::levels::Level;
use bevy_ldtk_scene::prelude::LevelMetaExt;
use bevy_light_2d::light::{AmbientLight2d, PointLight2d};
use bevy_optix::camera::MainCamera;
use bevy_seedling::prelude::Volume;
use bevy_seedling::prelude::*;
use bevy_seedling::sample::{PlaybackSettings, SamplePlayer};
use bevy_sequence::combinators::delay::run_after;
use bevy_sequence::prelude::FragmentExt;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, Interpolator};
use bevy_tween::tween::IntoTarget;
use bevy_tween::{BevyTweenRegisterSystems, component_tween_system};
use rand::Rng;

use crate::audio::{MusicPool, SpatialPool};
use crate::cutscene::fragments::IntoBox;
use crate::cutscenes::tea::fade_in_music;
use crate::interactions::Interacted;
use crate::inventory::item::{InventoryItem, ItemPickupEvent};
use crate::player::Player;
use crate::{Avian, Layer, world};

use super::{DoorDisabled, in_level};

pub struct BathroomPlugin;

impl Plugin for BathroomPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<world::Scribble, Scribble>()
            .add_tween_systems((
                component_tween_system::<AmbientLightTween>(),
                component_tween_system::<PointLight2dTween>(),
            ))
            .add_systems(
                Update,
                disable_bathroom_door.run_if(in_level(world::Level2.uid())),
            )
            .init_resource::<ScribbleDialogStep>()
            .add_systems(Avian, move_scribble)
            .add_observer(start)
            .add_observer(observe_scribbles);
    }
}

fn start(
    trigger: Trigger<OnAdd, Level>,
    levels: Query<&Level>,
    mut commands: Commands,
    camera: Single<Entity, With<MainCamera>>,
) {
    if !levels
        .get(trigger.target())
        .is_ok_and(|level| level.uid() == world::Level2.uid())
    {
        return;
    }

    let lights_duration = Duration::from_secs(2);

    commands
        .entity(*camera)
        .insert(AmbientLight2d::default())
        .animation()
        .insert_tween_here(
            lights_duration,
            EaseKind::Linear,
            camera
                .into_target()
                .with(AmbientLightTween { start: 1., end: 0. }),
        );

    run_after(
        lights_duration + Duration::from_secs(3),
        |mut commands: Commands, server: Res<AssetServer>| {
            sturgeon(&mut commands);
        },
        &mut commands,
    );
}

#[derive(Component)]
struct DisabledBathroomDoor;

fn disable_bathroom_door(
    mut commands: Commands,
    doors: Query<(Entity, &world::BathroomExitDoor), Without<DisabledBathroomDoor>>,
) {
    for (entity, _) in doors.iter() {
        commands
            .entity(entity)
            .insert((DoorDisabled, DisabledBathroomDoor));
    }
}

#[derive(Component)]
struct AmbientLightTween {
    start: f32,
    end: f32,
}

impl Interpolator for AmbientLightTween {
    type Item = AmbientLight2d;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.brightness = self.start.lerp(self.end, value);
    }
}

#[derive(Component)]
struct PointLight2dTween {
    start: f32,
    end: f32,
}

impl Interpolator for PointLight2dTween {
    type Item = PointLight2d;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.intensity = self.start.lerp(self.end, value);
    }
}

fn sturgeon(commands: &mut Commands) {
    crate::cutscenes::dark_home::sturgeon()
        .on_end(|mut commands: Commands| {
            run_after(
                Duration::from_secs(5),
                |mut commands: Commands,
                 server: Res<AssetServer>,
                 door: Query<Entity, With<DisabledBathroomDoor>>,
                 player: Single<Entity, With<Player>>| {
                    for entity in door.iter() {
                        commands
                            .entity(entity)
                            .remove::<DoorDisabled>()
                            .with_child((
                                PlaybackSettings {
                                    on_complete: bevy_seedling::sample::OnComplete::Remove,
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

                    let light = commands.spawn_empty().id();
                    commands
                        .entity(light)
                        .insert(PointLight2d {
                            intensity: 0.0,
                            radius: 25.,
                            ..Default::default()
                        })
                        .animation()
                        .insert_tween_here(
                            Duration::from_secs(3),
                            EaseKind::Linear,
                            light.into_target().with(PointLight2dTween {
                                start: 0.,
                                end: 0.2,
                            }),
                        );

                    commands.entity(*player).add_child(light);

                    commands.queue(|world: &mut World| world.run_system_once(fade_in_music(7.5)));
                    commands.spawn((
                        MusicPool,
                        SamplePlayer::new(server.load("audio/music/the-depths.ogg"))
                            .looping()
                            .with_volume(Volume::Decibels(-6.0)),
                    ));
                },
                &mut commands,
            );
        })
        .spawn_box(commands);
}

#[derive(Resource, Default)]
pub struct ScribbleDialogStep(usize);

#[derive(Component, Default)]
pub struct Exhausted;

#[derive(Component, Default)]
#[require(
    Collider::rectangle(16.0, 16.0),
    // RigidBody::Dynamic,
    // CollisionLayers::new(Layer::Default, Layer::Default),
    LockedAxes::ROTATION_LOCKED,
            Sensor,
    crate::interactions::Interactable
)]
#[component(on_insert = Self::on_insert_hook)]
pub struct Scribble;

impl Scribble {
    fn on_insert_hook(mut world: DeferredWorld, context: HookContext) {
        world.commands().queue(move |world: &mut World| {
            world.run_system_once(move |mut commands: Commands, server: Res<AssetServer>| {
                let mut rng = rand::thread_rng();
                commands.entity(context.entity).insert((
                    SpatialPool,
                    SamplePlayer::new(server.load("audio/sfx/solo-whispers.ogg"))
                        .looping()
                        .with_volume(Volume::Decibels(-6.0)),
                    PlaybackSettings {
                        playhead: Notify::new(Playhead::Seconds(rng.gen_range(0.0..5.0))),
                        speed: 0.9,
                        ..Default::default()
                    },
                    sample_effects![(SpatialScale(Vec3::splat(1.0)), SpatialBasicNode::default(),)],
                ));
            })
        });

        // world.commands().entity(context.entity).with_child((
        //     Collider::rectangle(16.0, 16.0),
        //     Transform::default(),
        //     Sensor,
        //     CollisionLayers::new(Layer::Default, [Layer::Player]),
        // ));
    }
}

fn move_scribble(mut scribbles: Query<&mut LinearVelocity, With<Scribble>>) {
    let mut rng = rand::thread_rng();
    for mut scribble in &mut scribbles {
        scribble.x = rng.gen_range(-1.0..1.0) * 5.0;
        scribble.y = rng.gen_range(-1.0..1.0) * 5.0;
    }
}

fn observe_scribbles(
    trigger: Trigger<OnAdd, Interacted>,
    scribbles: Query<Has<Exhausted>, With<Scribble>>,
    mut di: ResMut<ScribbleDialogStep>,
    mut commands: Commands,
) {
    let Ok(false) = scribbles.get(trigger.target()) else {
        return;
    };

    use crate::cutscenes::dark_home::*;
    match di.0 {
        0 => {
            shadow_1().spawn_box(&mut commands);
            di.0 += 1;
            commands.entity(trigger.target()).insert(Exhausted);
        }
        1 => {
            shadow_2().spawn_box(&mut commands);
            di.0 += 1;
            commands.entity(trigger.target()).insert(Exhausted);
        }
        2 => {
            shadow_3().spawn_box(&mut commands);
            di.0 += 1;
            commands.entity(trigger.target()).insert(Exhausted);
        }
        3 => {
            shadow_4().spawn_box(&mut commands);
            di.0 += 1;
            commands.entity(trigger.target()).insert(Exhausted);

            run_after(
                Duration::from_secs(1),
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
        }
        4 => {
            shadow_5().spawn_box(&mut commands);
            di.0 += 1;
            commands.entity(trigger.target()).insert(Exhausted);
        }
        5 => {
            shadow_6().spawn_box(&mut commands);
            di.0 += 1;
            commands.entity(trigger.target()).insert(Exhausted);
        }
        6 => {
            shadow_7().spawn_box(&mut commands);
            di.0 += 1;
            commands.entity(trigger.target()).insert(Exhausted);
        }
        7 => {
            shadow_8().spawn_box(&mut commands);
            di.0 += 1;
            commands.entity(trigger.target()).insert(Exhausted);
        }
        8 => {
            shadow_9()
                .on_end(
                    |mut commands: Commands, mut writer: EventWriter<ItemPickupEvent>| {
                        let item = commands
                            .spawn((InventoryItem {
                                name: "Key".into(),
                                description: "This seems important.".into(),
                            },))
                            .id();

                        writer.write(ItemPickupEvent(item));
                    },
                )
                .spawn_box(&mut commands);

            di.0 += 1;
            commands.entity(trigger.target()).insert(Exhausted);
        }
        _ => {}
    }
}
