#![allow(dead_code)]
use bevy::utils::HashMap;

/// The kind of item.
///
/// This is a unique identifier for the type of item.
struct ItemType(u64);

/// The data for as single [`ItemType`].
///
/// This is the data that is shared between all items of the same type.
struct ItemData {
    name: String,
    item_type: ItemType,
    description: String,
    value: i32,
    weight: f32,
    max_stack: i32,
}

/// A data-driven manifest, which contains all the data for all the items in the game.
struct ItemManifest {
    items: HashMap<ItemType, ItemData>,
}

struct ItemStack {
    item: ItemType,
    current_stack: i32,
}

fn main() {}
