use bevy::prelude::*;

/// Minimal information about an inventory item.
#[derive(Debug, Component)]
pub struct InventoryItem {
    pub name: String,
    pub description: String,
}

/// An event indicating the provided item entity was picked up.
#[derive(Debug, Component)]
pub struct ItemPickupEvent(Entity);
