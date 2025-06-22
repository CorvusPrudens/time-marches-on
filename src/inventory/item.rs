use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ItemPickupEvent>()
            .add_systems(Startup, |mut commands: Commands| {
                // commands.spawn(Inventory);

                commands.spawn((
                    Inventory,
                    children![
                        InventoryItem {
                            name: "Pencil".into(),
                            description: "You keep it on you at all times.".into(),
                        },
                        InventoryItem {
                            name: "Note (1)".into(),
                            description: "A note.".into(),
                        },
                    ],
                ));
            })
            .add_systems(PostUpdate, add_inventory_item);
    }
}

/// The player's inventory.
///
/// Inventory items are stored as children of this entity.
#[derive(Debug, Component)]
pub struct Inventory;

/// Minimal information about an inventory item.
#[derive(Debug, Component)]
pub struct InventoryItem {
    pub name: String,
    pub description: String,
}

/// An event indicating the provided item entity was picked up.
///
/// To add an item, make sure the entity has at least an [`InventoryItem`]
/// component.
///
/// ```
/// # use bevy::prelude::*;
/// fn add_item(mut writer: EventWriter<ItemPickupEvent>, mut commands: Commands) {
///     let item = commands.spawn(InventoryItem {
///         name: "My item".into(),
///         description: "My description...".into(),
///     }).id();
///
///     writer.write(ItemPickupEvent(item));
/// }
/// ```
#[derive(Debug, Event)]
pub struct ItemPickupEvent(pub Entity);

fn add_inventory_item(
    inventory: Single<Entity, With<Inventory>>,
    mut events: EventReader<ItemPickupEvent>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    for event in events.read() {
        commands.entity(*inventory).add_child(event.0);

        commands.spawn(SamplePlayer::new(server.load("audio/sfx/pickup.wav")));
    }
}
