use std::time::Duration;

use bevy::prelude::*;
use bevy_ldtk_scene::levels::Level;
use bevy_ldtk_scene::prelude::LevelMetaExt;
use bevy_seedling::prelude::*;
use bevy_sequence::combinators::delay::run_after;
use bevy_sequence::prelude::*;

use crate::audio::SpatialSound;
use crate::cutscene::fragments::IntoBox;
use crate::interactions::{Interactable, Interacted};
use crate::world;

pub struct VisitorPlugin;

impl Plugin for VisitorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<VisitorEvent>()
            .register_required_components::<world::FrontDoor, FrontDoor>()
            .add_systems(Update, (visitor, front_door_sprite))
            .add_observer(start)
            .add_observer(front_door);
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
        Duration::from_secs(0),
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

fn front_door(
    trigger: Trigger<OnAdd, Interacted>,
    front_door: Query<(Entity, &world::FrontDoor)>,

    mut commands: Commands,
    visitor: Single<(Entity, &VisitorEvent)>,
) {
    if front_door.get(trigger.target()).is_err() {
        return;
    };

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
    neighbor()
        .on_end(
            |mut commands: Commands, door: Single<Entity, With<TheDoor>>| {
                commands.entity(*door).insert((
                    DoorState::Closed,
                    crate::world::Interaction {
                        width: 32.,
                        height: 32.,
                        flavor: String::from("He is still outside..."),
                    },
                ));
            },
        )
        .spawn_box(commands);
}

fn neighbor() -> impl IntoBox {
    ("Hello, world!", "How are you?").always().once()
}
