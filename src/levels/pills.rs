use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ldtk_scene::levels::Level;
use bevy_ldtk_scene::prelude::LevelMetaExt;
use bevy_optix::zorder::YOrigin;
use bevy_seedling::prelude::*;
use bevy_sequence::combinators::delay::run_after;

use crate::interactions::{Interactable, Interacted};
use crate::inventory::item::{InventoryItem, ItemPickupEvent};
use crate::notes::NoteEvent;
use crate::textbox::{TextBlurb, TextboxEvent};
use crate::world::SideDoor1;
use crate::{Layer, world};

pub struct PillsPlugin;

impl Plugin for PillsPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<world::Pills, Pills>()
            .register_required_components::<world::PillTrash, Trash>()
            .register_required_components::<world::CrackedSideDoor1, CrackedDoor>()
            .add_observer(loop0)
            .add_observer(pills)
            .add_observer(trash);
    }
}

fn loop0(trigger: Trigger<OnAdd, Level>, levels: Query<&Level>, mut commands: Commands) {
    if !levels
        .get(trigger.target())
        .is_ok_and(|level| level.uid() == world::Level0.uid())
    {
        return;
    }

    commands.run_system_cached(crate::despawn_entities::<With<PillState>>);
    commands.spawn(PillState(0));

    run_after(
        Duration::from_secs(2),
        |mut writer: EventWriter<NoteEvent>| {
            writer.write(NoteEvent("pills1.png"));
        },
        &mut commands,
    );
}

#[derive(Component)]
struct PillState(usize);

#[derive(Default, Component)]
#[require(Visibility::Hidden, ColliderDisabled, YOrigin(-12.))]
struct CrackedDoor;

#[derive(Default, Component)]
#[require(
    Interactable,
    Collider::rectangle(24., 48.),
    CollisionLayers::new(Layer::Default, Layer::Player)
)]
struct Pills;

#[derive(Component)]
struct CollectedPills;

fn pills(
    trigger: Trigger<OnAdd, Interacted>,
    pills: Query<&world::Pills>,

    mut commands: Commands,
    mut writer: EventWriter<ItemPickupEvent>,
    mut textbox: EventWriter<TextboxEvent>,
) {
    let Ok(pills) = pills.get(trigger.target()) else {
        return;
    };

    textbox.write(TextboxEvent::section(TextBlurb::main_character(
        pills.flavor.clone(),
    )));

    commands.entity(trigger.target()).despawn();
    let item = commands
        .spawn((
            CollectedPills,
            InventoryItem {
                name: "Pills".into(),
                description: "Half full bottle of pills.".into(),
            },
        ))
        .id();
    writer.write(ItemPickupEvent(item));
}

#[derive(Default, Component)]
#[require(
    Interactable,
    Collider::rectangle(24., 24.),
    CollisionLayers::new(Layer::Default, Layer::Player)
)]
struct Trash;

fn trash(
    trigger: Trigger<OnAdd, Interacted>,
    trash: Query<&Trash>,
    pills: Query<Entity, With<CollectedPills>>,

    mut commands: Commands,
    server: Res<AssetServer>,
    mut writer: EventWriter<TextboxEvent>,

    id: Single<&PillState>,
) {
    if !trash.get(trigger.target()).is_ok() {
        return;
    }

    if pills.is_empty() {
        if id.0 == 0 {
            writer.write(TextboxEvent::section(TextBlurb::main_character(
                "Where are the pills?",
            )));
        }

        return;
    }

    commands.spawn(SamplePlayer {
        sample: server.load("audio/sfx/pills.wav"),
        volume: Volume::Linear(1.25),
        ..Default::default()
    });

    for entity in pills.iter() {
        commands.entity(entity).despawn();
    }

    commands.entity(trigger.target()).despawn();

    run_after(
        Duration::from_secs(1),
        |mut commands: Commands,
         server: Res<AssetServer>,
         cracked_door: Query<(Entity, &world::CrackedSideDoor1)>,
         side_door: Query<(Entity, &SideDoor1)>,
         mut id: Single<&mut PillState>| {
            for (entity, _) in side_door
                .iter()
                .filter(|(_, door)| door.id as usize == id.0)
            {
                commands.entity(entity).despawn();
            }
            for (entity, _) in cracked_door
                .iter()
                .filter(|(_, door)| door.id as usize == id.0)
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

            id.0 += 1;
        },
        &mut commands,
    );
}
