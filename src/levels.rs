use avian2d::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_optix::camera::MainCamera;

use crate::player::Player;
use crate::{GameState, HexColor, Layer, TILE_SIZE, world};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<world::Teleport, Teleporter>()
            .add_systems(Update, add_tile_collision)
            .add_systems(OnEnter(GameState::Playing), load_ldtk)
            .add_observer(teleport);
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

fn load_ldtk(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut camera: Single<&mut Camera, With<MainCamera>>,
) {
    camera.clear_color = ClearColorConfig::Custom(HexColor(0x252525).into());
    commands.spawn((
        //bevy_ldtk_scene::HotWorld(server.load("ldtk/time-marches-on.ldtk")),
        bevy_ldtk_scene::World(server.load("ldtk/time-marches-on.ron")),
        bevy_ldtk_scene::prelude::LevelLoader::levels((world::Level0, world::Level1)),
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
