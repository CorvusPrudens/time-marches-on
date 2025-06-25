use std::time::Duration;

use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_enhanced_input::events::Fired;
use bevy_enhanced_input::prelude::Actions;
use bevy_ldtk_scene::levels::LevelLoader;
use bevy_optix::camera::MainCamera;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_seedling::prelude::Volume;
use bevy_seedling::sample::SamplePlayer;
use bevy_sequence::combinators::delay::run_after;
use bevy_tween::combinator::{sequence, tween};
use bevy_tween::interpolate::sprite_color_to;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind};
use bevy_tween::tween::IntoTarget;

use crate::callback::Callback;
use crate::interactions::{InteractAction, Interactable};
use crate::player::{Player, PlayerCollider, PlayerContext};
use crate::textbox::{TextBlurb, TextboxEvent};
use crate::{GameState, HexColor, Layer, TILE_SIZE, world};

mod pills;
mod visitor;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((pills::PillsPlugin, visitor::VisitorPlugin))
            .register_required_components::<world::Teleport, Teleporter>()
            .register_required_components::<world::LunaDoor, VerticalDoor>()
            .register_required_components::<world::HallDoor1, VerticalDoor>()
            .register_required_components::<world::FrontDoor, VerticalDoor>()
            .register_required_components::<world::SideDoor1, Door>()
            .register_required_components::<world::SideDoor2, Door>()
            .register_required_components::<world::BathroomDoor, VerticalDoor>()
            .register_required_components::<world::BathroomExitDoor, VerticalDoor>()
            .register_required_components::<world::CrackedSideDoor1, Door>()
            .add_systems(Update, (add_tile_collision, manage_transitions))
            .add_systems(OnEnter(GameState::Playing), load_ldtk)
            .add_observer(teleport)
            .add_observer(door);
    }
}

#[derive(Default, Component)]
#[require(
    Collider::rectangle(8., 256.),
    Sensor,
    CollisionEventsEnabled,
    CollisionLayers::new(Layer::Default, Layer::Player)
)]
struct Teleporter;

fn teleport(
    trigger: Trigger<OnCollisionEnd>,
    teleporter: Query<&world::Teleport>,
    player: Single<(&mut Transform, &LinearVelocity), With<Player>>,
) {
    let Ok(teleport) = teleporter.get(trigger.target()) else {
        return;
    };

    let (mut transform, velocity) = player.into_inner();
    let diff = if velocity.x.is_sign_positive() {
        teleport.forward
    } else {
        -teleport.backward
    };

    transform.translation.x += diff;
}

#[derive(Default)]
enum TransitionStage {
    #[default]
    In,
    Out,
}

#[derive(Component)]
#[component(on_remove = Self::on_remove_hook, on_insert = Self::on_insert_hook)]
struct ScreenTransition {
    duration: Duration,
    timer: Timer,
    stage: TransitionStage,
    on_black: Callback,
    on_complete: Callback,
}

impl ScreenTransition {
    pub fn new<S1, M1, S2, M2>(duration: Duration, on_black: S1, on_complete: S2) -> Self
    where
        S1: IntoSystem<(), (), M1> + Send + Sync + 'static,
        M1: 'static,
        S2: IntoSystem<(), (), M2> + Send + Sync + 'static,
        M2: 'static,
    {
        Self {
            duration,
            timer: Timer::new(duration, TimerMode::Once),
            stage: TransitionStage::In,
            on_black: Callback::new(on_black),
            on_complete: Callback::new(on_complete),
        }
    }

    fn on_insert_hook(mut world: DeferredWorld, context: HookContext) {
        let duration = world.get::<Self>(context.entity).unwrap().duration;
        let mut commands = world.commands();
        let mut entity = commands.entity(context.entity);

        entity.insert((
            HIGH_RES_LAYER,
            Sprite::from_color(Color::NONE, Vec2::new(crate::WIDTH, crate::HEIGHT)),
            Transform::from_translation(Vec3::new(0.0, 0.0, 999.0))
                .with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
        ));

        let target = entity.id().into_target();
        let mut color = target.state(Color::NONE);

        entity.animation().insert(sequence((
            tween(
                duration,
                EaseKind::QuadraticOut,
                color.with(sprite_color_to(Color::BLACK)),
            ),
            tween(
                duration,
                EaseKind::QuadraticIn,
                color.with(sprite_color_to(Color::NONE)),
            ),
        )));
    }

    fn on_remove_hook(mut world: DeferredWorld, context: HookContext) {
        let trans = world.get::<ScreenTransition>(context.entity).unwrap();

        let a = trans.on_black.0.clone();
        let b = trans.on_complete.0.clone();

        world.commands().queue(move |world: &mut World| -> Result {
            a.lock().unwrap().unregister(world)?;
            b.lock().unwrap().unregister(world)?;

            Ok(())
        });
    }
}

fn manage_transitions(
    mut transitions: Query<(Entity, &mut ScreenTransition)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta = time.delta();

    for (entity, mut transition) in transitions.iter_mut() {
        if transition.timer.tick(delta).just_finished() {
            match transition.stage {
                TransitionStage::In => {
                    let on_black = transition.on_black.0.clone();
                    commands
                        .queue(move |world: &mut World| on_black.lock().unwrap().call(world, ()));

                    transition.stage = TransitionStage::Out;
                    transition.timer = Timer::new(transition.duration, TimerMode::Once);
                }
                TransitionStage::Out => {
                    let on_complete = transition.on_complete.0.clone();
                    commands.queue(move |world: &mut World| {
                        on_complete.lock().unwrap().call(world, ())
                    });

                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[derive(Default, Component)]
#[require(
    Collider::rectangle(16., 24.),
    Sensor,
    CollidingEntities,
    CollisionLayers::new(Layer::Default, Layer::Player)
)]
struct VerticalDoor;

#[derive(Default, Component)]
#[require(
    Collider::rectangle(24., 24.),
    Sensor,
    CollidingEntities,
    CollisionLayers::new(Layer::Default, Layer::Player),
    Interactable
)]
struct Door;

fn door(
    _: Trigger<Fired<InteractAction>>,

    luna_door: Query<(&world::LunaDoor, &CollidingEntities, &ChildOf)>,
    hall_doors1: Query<(&world::HallDoor1, &CollidingEntities, &ChildOf)>,
    hall_doors2: Query<(&world::FrontDoor, &CollidingEntities, &ChildOf)>,
    side_doors1: Query<(&world::SideDoor1, &CollidingEntities, &ChildOf)>,
    side_doors2: Query<(&world::SideDoor2, &CollidingEntities, &ChildOf)>,
    bathroom_door1: Query<(&world::BathroomDoor, &CollidingEntities, &ChildOf)>,
    bathroom_door2: Query<(&world::BathroomExitDoor, &CollidingEntities, &ChildOf)>,
    cracked_side_door1: Query<(&world::CrackedSideDoor1, &CollidingEntities, &ChildOf)>,

    player: Single<(Entity, &mut Transform), With<Player>>,
    player_collider: Single<Entity, With<PlayerCollider>>,
    transforms: Query<&GlobalTransform>,
    mut writer: EventWriter<TextboxEvent>,

    mut commands: Commands,
    server: Res<AssetServer>,

    mut loader: Single<&mut LevelLoader>,
) -> Result {
    for (target, child_of, luna, load) in bathroom_door1
        .iter()
        .map(|(door, colliding, child_of)| (door.target, colliding, child_of, false, ""))
        .chain(
            bathroom_door2
                .iter()
                .map(|(door, colliding, child_of)| (door.target, colliding, child_of, false, "")),
        )
        .chain(
            luna_door
                .iter()
                .map(|(door, colliding, child_of)| (door.target, colliding, child_of, true, "")),
        )
        .chain(
            side_doors1
                .iter()
                .map(|(door, colliding, child_of)| (door.target, colliding, child_of, false, "")),
        )
        .chain(
            side_doors2
                .iter()
                .map(|(door, colliding, child_of)| (door.target, colliding, child_of, false, "")),
        )
        .chain(hall_doors1.iter().map(|(door, colliding, child_of)| {
            let level_t = transforms.get(child_of.parent()).unwrap().translation();

            (
                door.target.or_else(|| {
                    (door.x != 0.0 && door.y != 0.0).then_some(Vec2::new(
                        (door.x - level_t.x) / 16.,
                        -(-door.y - level_t.y) / 16.,
                    ))
                }),
                colliding,
                child_of,
                false,
                door.load.as_str(),
            )
        }))
        .chain(
            hall_doors2
                .iter()
                .map(|(door, colliding, child_of)| (door.target, colliding, child_of, false, "")),
        )
        .chain(
            cracked_side_door1
                .iter()
                .map(|(door, colliding, child_of)| (door.target, colliding, child_of, false, "")),
        )
        .filter_map(|(target, colliding, child_of, luna, load)| {
            colliding
                .contains(&*player_collider)
                .then_some((target, child_of, luna, load))
        })
    {
        if !load.is_empty() {
            match load {
                "level1" => {
                    loader.spawn(world::Level1);
                    run_after(
                        Duration::from_secs(2),
                        |mut loader: Single<&mut LevelLoader>| {
                            loader.despawn(world::Level0);
                        },
                        &mut commands,
                    );
                }
                _ => panic!("{}", load),
            }
        }

        match target {
            Some(target) => {
                let level_t = transforms.get(child_of.parent())?.translation();

                commands.entity(player.0).remove::<Actions<PlayerContext>>();
                commands.spawn(
                    SamplePlayer::new(server.load("audio/sfx/door.wav"))
                        .with_volume(Volume::Decibels(-12.0)),
                );

                commands.spawn(ScreenTransition::new(
                    Duration::from_millis(250),
                    move |mut player: Single<&mut Transform, With<Player>>| {
                        player.translation.x = target.x * 16. + level_t.x;
                        player.translation.y = -target.y * 16. + level_t.y;
                    },
                    |player: Single<Entity, With<Player>>, mut commands: Commands| {
                        commands
                            .entity(*player)
                            .insert(Actions::<PlayerContext>::default());
                    },
                ));
            }
            None => {
                commands.spawn(SamplePlayer {
                    sample: server.load("audio/sfx/door-handle.wav"),
                    volume: Volume::Linear(0.5),
                    ..Default::default()
                });
                if luna {
                    writer.write(TextboxEvent::section(TextBlurb::narrator("Locked...")));
                }
            }
        }
    }

    Ok(())
}

fn load_ldtk(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut camera: Single<&mut Camera, With<MainCamera>>,
) {
    camera.clear_color = ClearColorConfig::Custom(HexColor(0x252525).into());
    commands.spawn((
        bevy_ldtk_scene::HotWorld(server.load("ldtk/time-marches-on.ldtk")),
        bevy_ldtk_scene::World(server.load("ldtk/time-marches-on.ron")),
        bevy_ldtk_scene::prelude::LevelLoader::levels(world::Level0),
    ));
}

fn add_tile_collision(
    mut commands: Commands,
    tiles: Query<(&Transform, &ChildOf, &world::Tile), Added<world::Tile>>,
) {
    if tiles.is_empty() {
        return;
    }

    let mut level_tiles = HashMap::<Entity, Vec<_>>::default();
    let tile_size = TILE_SIZE;
    let offset = tile_size / 2.;
    for (t, c, _) in tiles
        .iter()
        .filter(|(_, _, t)| matches!(t, world::Tile::Collision))
    {
        level_tiles.entry(c.parent()).or_default().push(Vec2::new(
            t.translation.x + offset,
            t.translation.y + offset,
        ));
    }

    if level_tiles.is_empty() {
        return;
    }

    for (entity, tiles) in level_tiles.into_iter() {
        commands.entity(entity).with_children(|level| {
            for (pos, collider) in build_colliders_from_vec2(tiles, tile_size).into_iter() {
                level.spawn((
                    Transform::from_translation((pos - Vec2::splat(tile_size / 2.)).extend(0.)),
                    RigidBody::Static,
                    collider,
                ));
            }
        });
    }
}

fn build_colliders_from_vec2(mut positions: Vec<Vec2>, tile_size: f32) -> Vec<(Vec2, Collider)> {
    positions.sort_by(|a, b| {
        let y_cmp = a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal);
        if y_cmp == std::cmp::Ordering::Equal {
            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            y_cmp
        }
    });

    let mut rows = Vec::with_capacity(positions.len() / 2);
    let mut current_y = None;
    let mut current_xs = Vec::with_capacity(positions.len() / 2);
    for v in positions.into_iter() {
        match current_y {
            None => {
                current_y = Some(v.y);
                current_xs.push(v.x);
            }
            Some(y) => {
                if v.y == y {
                    current_xs.push(v.x);
                } else {
                    rows.push((y, current_xs.clone()));
                    current_xs.clear();

                    current_y = Some(v.y);
                    current_xs.push(v.x);
                }
            }
        }
    }

    match current_y {
        Some(y) => {
            rows.push((y, current_xs));
        }
        None => unreachable!(),
    }

    #[derive(Debug, Clone, Copy)]
    struct Plate {
        y: f32,
        x_start: f32,
        x_end: f32,
    }

    let mut row_plates = Vec::with_capacity(rows.len());
    for (y, row) in rows.into_iter() {
        let mut current_x = None;
        let mut x_start = None;
        let mut plates = Vec::with_capacity(row.len() / 4);

        for x in row.iter() {
            match (current_x, x_start) {
                (None, None) => {
                    current_x = Some(*x);
                    x_start = Some(*x);
                }
                (Some(cx), Some(xs)) => {
                    if *x > cx + tile_size {
                        plates.push(Plate {
                            x_end: cx + tile_size,
                            x_start: xs,
                            y,
                        });
                        x_start = Some(*x);
                    }

                    current_x = Some(*x);
                }
                _ => unreachable!(),
            }
        }

        match (current_x, x_start) {
            (Some(cx), Some(xs)) => {
                plates.push(Plate {
                    x_end: cx + tile_size,
                    x_start: xs,
                    y,
                });
            }
            _ => unreachable!(),
        }

        row_plates.push(plates);
    }

    let mut output = Vec::new();
    for plates in row_plates.into_iter() {
        for plate in plates.into_iter() {
            output.push((
                Vec2::new(
                    plate.x_end - (plate.x_end - plate.x_start) / 2.,
                    plate.y - tile_size / 2.,
                ),
                Collider::rectangle(plate.x_end - plate.x_start, tile_size),
            ));
        }
    }

    output
}
