//! This example demonstrates the simplest use of the `leafwing_manifest` crate.
//!
//! In this example, the manifest and raw manifest are the same type, and the data is read directly from the serialized format on disk into the [`ItemManifest`] resource.
//! This pattern is great for simple prototyping and small projects, but can be quickly outgrown as the project's needs scale.
//! See the other examples for more advanced use cases!

use bevy::{prelude::*, utils::HashMap};
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
    plugin::{AppExt, ManifestPlugin},
};
use serde::{Deserialize, Serialize};

/// The data for as single item that might be held in the player's inventory.
///
/// All items with the same name have the same [`Item`] data:
/// a sword of slaying is always a sword of slaying, no matter how many swords the player has.
///
/// Tracking the number of items the player has is done elsewhere, in the player's inventory.
/// Per-item data, such as durability or enchantments, would also be tracked elsewhere.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)] // Properties are for demonstration purposes only.
struct Item {
    name: String,
    description: String,
    value: i32,
    weight: f32,
    max_stack: u8,
}

/// A data-driven manifest, which contains the canonical data for all the items in the game.
#[derive(Debug, Resource, Asset, TypePath, Serialize, Deserialize, PartialEq)]
struct ItemManifest {
    items: HashMap<Id<Item>, Item>,
}

impl Manifest for ItemManifest {
    // Because we're not doing any conversion between the raw and final data,
    // we can use the same type for both.
    type Item = Item;
    type RawItem = Item;
    // Similarly, we don't need to do any conversion between the raw and final data.
    type RawManifest = ItemManifest;
    // Converting between the raw and final data is trivial, so we can use `Infallible`.
    type ConversionError = std::convert::Infallible;

    const FORMAT: ManifestFormat = ManifestFormat::Ron;

    fn get(&self, id: &Id<Item>) -> Option<&Self::Item> {
        self.items.get(id)
    }

    // We're able to read the data directly from the serialized format,
    // so there's no need for any intermediate conversion.
    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        _world: &mut World,
    ) -> Result<Self, Self::ConversionError> {
        Ok(raw_manifest)
    }
}

fn main() {
    App::new()
        // Default plugins contain `AssetPlugin`, which is required for asset loading.
        .add_plugins(DefaultPlugins)
        // This is our simple state, used to navigate the asset loading process.
        .init_state::<SimpleAssetState>()
        // Coordinates asset loading and state transitions.
        .add_plugins(ManifestPlugin::<SimpleAssetState>::default())
        // Registers our item manifest, triggering it to be loaded.
        .register_manifest::<ItemManifest>("items.ron")
        .add_systems(
            Update,
            list_available_items
                .run_if(run_once())
                .run_if(in_state(SimpleAssetState::Ready)),
        );
}

/// This system reads the generated item manifest resource and prints out all the items.
fn list_available_items(item_manifest: Res<ItemManifest>) {
    for (id, item) in item_manifest.items.iter() {
        println!("{:?}: {:?}", id, item);
    }
}

/// This module is used to generate the item manifest.
///
/// While manifests *can* be hand-authored, it's often more convenient to generate them using tooling of some kind.
/// Serde's [`Serialize`] and [`Deserialize`] traits are a good fit for this purpose.
/// `ron` is a straightforward human-readable format that plays well with Rust's type system, and is a good point to start.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_item_manifest() {
        let mut items = HashMap::default();

        items.insert(
            Id::from_name("sword".into()),
            Item {
                name: "sword".into(),
                description: "A sharp sword".into(),
                value: 10,
                weight: 2.0,
                max_stack: 1,
            },
        );

        items.insert(
            Id::from_name("shield".into()),
            Item {
                name: "shield".into(),
                description: "A sturdy shield".into(),
                value: 5,
                weight: 5.0,
                max_stack: 1,
            },
        );

        let item_manifest = ItemManifest { items };

        let serialized = ron::ser::to_string_pretty(&item_manifest, Default::default()).unwrap();
        println!("{}", serialized);

        // Save the results, to ensure that our example has a valid manifest to read.
        std::fs::write("assets/items.ron", &serialized).unwrap();

        let deserialized: ItemManifest = ron::de::from_str(&serialized).unwrap();

        assert_eq!(item_manifest, deserialized);
    }
}
