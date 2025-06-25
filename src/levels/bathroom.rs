use std::time::Duration;

use bevy::prelude::*;
use bevy_ldtk_scene::levels::Level;
use bevy_ldtk_scene::prelude::LevelMetaExt;
use bevy_light_2d::light::{AmbientLight2d, PointLight2d};
use bevy_optix::camera::MainCamera;
use bevy_seedling::sample::{PlaybackSettings, SamplePlayer};
use bevy_sequence::combinators::delay::run_after;
use bevy_sequence::prelude::FragmentExt;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, Interpolator};
use bevy_tween::tween::IntoTarget;
use bevy_tween::{BevyTweenRegisterSystems, component_tween_system};

use crate::cutscene::fragments::IntoBox;
use crate::player::Player;
use crate::world;

use super::{DoorDisabled, in_level};

pub struct BathroomPlugin;

impl Plugin for BathroomPlugin {
    fn build(&self, app: &mut App) {
        app.add_tween_systems((
            component_tween_system::<AmbientLightTween>(),
            component_tween_system::<PointLight2dTween>(),
        ))
        .add_systems(
            Update,
            disable_bathroom_door.run_if(in_level(world::Level2.uid())),
        )
        .add_observer(start);
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

    commands
        .entity(*camera)
        .insert(AmbientLight2d::default())
        .animation()
        .insert_tween_here(
            Duration::from_secs(20),
            EaseKind::Linear,
            camera
                .into_target()
                .with(AmbientLightTween { start: 1., end: 0. }),
        );

    run_after(
        Duration::from_secs(25),
        |mut commands: Commands| {
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
                                crate::audio::SpatialSound,
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
                },
                &mut commands,
            );
        })
        .spawn_box(commands);
}
