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
use bevy_optix::zorder::YOrigin;
use bevy_seedling::prelude::Volume;
use bevy_seedling::prelude::*;
use bevy_seedling::sample::{PlaybackSettings, SamplePlayer};
use bevy_sequence::combinators::delay::run_after;
use bevy_sequence::prelude::FragmentExt;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, Interpolator};
use bevy_tween::tween::IntoTarget;
use bevy_tween::{BevyTweenRegisterSystems, component_tween_system};
use rand::Rng;

use crate::animation::{AnimationAppExt, AnimationSprite};
use crate::audio::{MusicPool, SpatialPool};
use crate::cutscene::fragments::IntoBox;
use crate::cutscenes::dark_home::final_cutscene;
use crate::cutscenes::tea::{fade_in_music, fade_out_music};
use crate::interactions::{Interactable, Interacted};
use crate::inventory::item::{InventoryItem, ItemPickupEvent};
use crate::player::Player;
use crate::{Avian, world};

use super::{DoorDisabled, in_level};

pub struct BathroomPlugin;

impl Plugin for BathroomPlugin {
    fn build(&self, app: &mut App) {
        app.register_layout(
            "textures/scribble.png",
            TextureAtlasLayout::from_grid(UVec2::splat(48), 4, 1, None, None),
        )
        .register_required_components::<world::Scribble, Scribble>()
        .register_required_components::<world::LunaDoor, LunaDoor>()
        .register_required_components::<world::BedEntity, BedEntity>()
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
        .add_observer(observe_scribbles)
        .add_observer(observe_door)
        .add_observer(observe_bed);
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

    let lights_duration = Duration::from_secs(10);

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

    commands.queue(|world: &mut World| world.run_system_once(fade_out_music(0.1)));

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
    Collider::rectangle(24.0, 32.0),
    YOrigin(-12.),
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
                    AnimationSprite::repeating("textures/scribble.png", 0.2, 0..4),
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
                    |mut commands: Commands,
                     mut writer: EventWriter<ItemPickupEvent>,
                     door: Single<Entity, With<LunaDoor>>| {
                        let item = commands
                            .spawn((InventoryItem {
                                name: "Key".into(),
                                description: "This seems important.".into(),
                            },))
                            .id();

                        writer.write(ItemPickupEvent(item));

                        commands.entity(*door).remove::<DoorDisabled>();
                    },
                )
                .spawn_box(&mut commands);

            di.0 += 1;
            commands.entity(trigger.target()).insert(Exhausted);
        }
        _ => {}
    }
}

#[derive(Component, Default)]
#[require(DoorDisabled, Collider::rectangle(16.0, 32.0), Interactable)]
pub struct LunaDoor;

fn observe_door(
    trigger: Trigger<OnAdd, Interacted>,
    door: Query<(), With<LunaDoor>>,
    mut commands: Commands,
) {
    if door.get(trigger.target()).is_err() {
        return;
    }

    commands.queue(|world: &mut World| world.run_system_once(fade_out_music(0.1)));
}

#[derive(Component, Default)]
#[require(Collider::rectangle(32.0, 64.0), Interactable)]
pub struct BedEntity;

fn observe_bed(
    trigger: Trigger<OnAdd, Interacted>,
    bed: Query<(), With<BedEntity>>,
    mut commands: Commands,
) {
    if bed.get(trigger.target()).is_err() {
        return;
    }

    final_cutscene().spawn_box(&mut commands);
}
