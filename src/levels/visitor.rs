use std::time::Duration;

use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_ldtk_scene::levels::Level;
use bevy_ldtk_scene::prelude::LevelMetaExt;
use bevy_light_2d::light::{AmbientLight2d, PointLight2d};
use bevy_optix::camera::{MainCamera, PixelSnap};
use bevy_optix::post_process::PostProcessCommand;
use bevy_optix::zorder::YOrigin;
use bevy_seedling::prelude::*;
use bevy_sequence::combinators::delay::run_after;
use bevy_sequence::prelude::*;

use crate::animation::{AnimationAppExt, AnimationSprite};
use crate::audio::SpatialSound;
use crate::cutscene::fragments::IntoBox;
use crate::interactions::{Interactable, Interacted};
use crate::player::{PLAYER_SPEED, Player, Scaled};
use crate::{Layer, world};

pub struct VisitorPlugin;

impl Plugin for VisitorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<VisitorEvent>()
            .register_required_components::<world::FrontDoor, FrontDoor>()
            .register_required_components::<world::LunaRun, LunaRun>()
            .register_required_components_with::<world::Bush, _>(|| YOrigin(-24.))
            .register_required_components::<world::Tree, Tree>()
            .register_required_components_with::<world::TreeTrunk, _>(|| YOrigin(-24.))
            .register_required_components::<world::TreeMan, TreeMan>()
            .register_layout(
                "textures/luna.png",
                TextureAtlasLayout::from_grid(UVec2::splat(48), 12, 8, None, None),
            )
            .add_systems(Update, (visitor, front_door_sprite, luna_run))
            .add_observer(start)
            .add_observer(front_door)
            .add_observer(tree_talk)
            .add_observer(side_door);
    }
}

fn start(trigger: Trigger<OnAdd, Level>, levels: Query<&Level>, mut commands: Commands) {
    if !levels
        .get(trigger.target())
        .is_ok_and(|level| level.uid() == world::Level1.uid())
    {
        return;
    }

    run_after(
        Duration::from_secs(5),
        |mut commands: Commands,
         server: Res<AssetServer>,
         mut writer: EventWriter<VisitorEvent>| {
            writer.write(VisitorEvent(friendly_neighbor));
            commands.spawn((
                Knocking,
                SamplePlayer::new(server.load("audio/sfx/knocking.wav"))
                    .looping()
                    .with_volume(Volume::Linear(0.9)),
                Transform::from_xyz(1400., -2291., 0.),
                SpatialSound,
            ));
        },
        &mut commands,
    );
}

fn side_door(
    trigger: Trigger<OnAdd, Interacted>,
    side_door: Query<&world::CrackedSideDoor1>,

    tree: Single<&GlobalTransform, With<world::Tree>>,

    mut commands: Commands,
    server: Res<AssetServer>,
) {
    if !side_door
        .get(trigger.target())
        .is_ok_and(|door| door.id == 8392)
    {
        return;
    };

    let position = tree.translation();
    commands.spawn((
        Transform::from_translation(position - Vec3::Y * 40.),
        PointLight2d {
            intensity: 0.6,
            radius: 150.,
            ..Default::default()
        },
    ));

    commands.post_process::<MainCamera>(AmbientLight2d {
        brightness: 0.1,
        ..Default::default()
    });

    commands.spawn((
        NightSfx,
        SamplePlayer::new(server.load("audio/sfx/night.ogg"))
            .looping()
            .with_volume(Volume::Linear(0.4)),
    ));
}

#[derive(Default, Component)]
#[require(YOrigin(-64.))]
struct Tree;

#[derive(Default, Component)]
#[require(
    YOrigin(-12.),
    Scaled(Vec2::splat(0.8)),
    PixelSnap,
)]
#[component(on_add = Self::init)]
struct LunaRun;

impl LunaRun {
    fn init(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .entity(ctx.entity)
            .insert(AnimationSprite::repeating("textures/luna.png", 0.0, [37]));
    }
}

fn luna_run(
    mut commands: Commands,
    luna: Single<(Entity, &GlobalTransform), With<LunaRun>>,
    player: Single<&Transform, With<Player>>,
) {
    let (entity, gt) = luna.into_inner();

    if gt.translation().xy().distance(player.translation.xy()) < 80. {
        commands
            .entity(entity)
            .insert((
                RigidBody::Kinematic,
                LinearVelocity(Vec2::X * PLAYER_SPEED * 1.25),
                AnimationSprite::repeating("textures/luna.png", 0.15, [12, 14]),
            ))
            .remove::<LunaRun>();
    }
}

#[derive(Default, Component)]
#[require(
    Interactable,
    Collider::rectangle(24., 48.),
    CollisionLayers::new(Layer::Default, Layer::Player)
)]
struct TreeMan;

fn tree_talk(
    trigger: Trigger<OnAdd, Interacted>,
    tree_man: Query<&world::TreeMan>,

    mut commands: Commands,
) {
    if tree_man.get(trigger.target()).is_err() {
        return;
    };

    commands.entity(trigger.target()).despawn();
    tree_man_scene(&mut commands);
}

fn tree_man_scene(commands: &mut Commands) {
    tree_man()
        .on_end(|mut commands: Commands| {
            run_after(
                Duration::from_secs(5),
                |mut commands: Commands,
                 server: Res<AssetServer>,
                 cracked_door: Query<(Entity, &world::CrackedSideDoor1)>| {
                    for (entity, _) in cracked_door
                        .iter()
                        .filter(|(_, door)| door.id as usize == 222)
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
                                crate::audio::SpatialSound,
                            ));
                    }
                },
                &mut commands,
            );
        })
        .spawn_box(commands);
}

fn tree_man() -> impl IntoBox {
    ("Hello, world!", "How are you?").always().once()
}

#[derive(Component)]
struct Knocking;

#[derive(Default, Component)]
#[require(Interactable, DoorState)]
struct FrontDoor;

#[derive(Default, Component)]
enum DoorState {
    #[default]
    Closed,
    Open,
}

fn front_door_sprite(
    mut commands: Commands,
    server: Res<AssetServer>,
    entities: Query<(Entity, &DoorState), Changed<DoorState>>,
) {
    for (entity, state) in entities.iter() {
        match state {
            DoorState::Closed => {
                commands.entity(entity).insert(Sprite {
                    image: server.load("textures/front-door.png"),
                    rect: Some(Rect::from_corners(Vec2::new(0., 0.), Vec2::new(16., 32.))),
                    ..Default::default()
                });
            }
            DoorState::Open => {
                commands.entity(entity).insert(Sprite {
                    image: server.load("textures/front-door.png"),
                    rect: Some(Rect::from_corners(Vec2::new(16., 0.), Vec2::new(32., 32.))),
                    ..Default::default()
                });
            }
        }
    }
}

#[derive(Component)]
struct TheDoor;

#[derive(Component)]
struct NightSfx;

fn front_door(
    trigger: Trigger<OnAdd, Interacted>,
    front_door: Query<(Entity, &world::FrontDoor)>,

    mut commands: Commands,
    server: Res<AssetServer>,
    visitor: Single<(Entity, &VisitorEvent)>,
) {
    if front_door.get(trigger.target()).is_err() {
        return;
    };

    commands.spawn((
        NightSfx,
        SamplePlayer::new(server.load("audio/sfx/night.ogg"))
            .looping()
            .with_volume(Volume::Linear(0.4)),
    ));

    commands.run_system_cached(crate::despawn_entities::<With<Knocking>>);
    commands
        .entity(trigger.target())
        .insert((DoorState::Open, TheDoor));

    let (entity, visitor) = visitor.into_inner();
    commands.entity(entity).despawn();

    visitor.0(&mut commands);
}

#[derive(Clone, Copy, Event, Component)]
struct VisitorEvent(fn(&mut Commands));

fn visitor(mut commands: Commands, mut reader: EventReader<VisitorEvent>) {
    debug_assert!(reader.len() <= 1);

    for event in reader.read() {
        commands.spawn(*event);
    }
}

fn friendly_neighbor(commands: &mut Commands) {
    crate::cutscenes::visitor::visitor()
        .on_end(
            |mut commands: Commands,
             door: Single<Entity, With<TheDoor>>,
             server: Res<AssetServer>| {
                commands.entity(*door).insert((
                    DoorState::Closed,
                    crate::world::Interaction {
                        width: 32.,
                        height: 32.,
                        flavor: String::from("He is still outside..."),
                    },
                ));

                commands.spawn(SamplePlayer::new(server.load("audio/sfx/door-close.wav")));
                commands.run_system_cached(crate::despawn_entities::<With<NightSfx>>);

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
                                    crate::audio::SpatialSound,
                                ));
                        }
                    },
                    &mut commands,
                );
            },
        )
        .spawn_box(commands);
}
