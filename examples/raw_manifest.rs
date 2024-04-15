//! When working with manifests, you may find that the most convenient format for serialization is not the most convenient format for working with in code.
//!
//! For example, you may need to cross-reference other items in the manifest, perform some kind of validation on the data, or store handles to assets.
//! While we *could* perform these operations every time we need to access the data, it's more efficient to perform them once and store the results.
//!
//! This is where raw manifests come in.
//! Serialized data is converted to raw manifests via asset loading,
//! and then processed into the final manifests which can be accessed as resources to look up the properties of your items.
//!
//! This example showcases a relatively simple raw manifest pattern, where the raw manifest is used to initialize handles to sprites;
//! see the other examples for more complex use cases!
//! This example builds on the `simple.rs` example, using much of the same code and patterns.

use std::path::PathBuf;

use bevy::{app::AppExit, prelude::*, utils::HashMap};
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
    plugin::{AppExt, ManifestPlugin},
};
use serde::{Deserialize, Serialize};

/// The data for as single item that might be held in the player's inventory.
///
/// This is the format that our item data is stored in after it's been loaded into a Bevy [`Resource`].
#[derive(Debug, PartialEq)]
#[allow(dead_code)] // Properties are for demonstration purposes only.
struct Item {
    name: String,
    description: String,
    value: i32,
    weight: f32,
    max_stack: u8,
    sprite: Handle<Image>,
}

/// The raw format for [`Item`] data.
///
/// This is used inside of [`RawItemManifest`] to be saved/loaded to disk as an [`Asset`].
/// The only difference in this case is that the `sprite` field has been changed from a loaded [`Handle<Image>`] to a [`PathBuf`].
/// This [`PathBuf`] references the actual sprite path in our assets folder,
/// but other identifiers could be used for more complex asset loading strategies.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct RawItem {
    name: String,
    description: String,
    value: i32,
    weight: f32,
    max_stack: u8,
    sprite: PathBuf,
}

/// A data-driven manifest, which contains the canonical data for all the items in the game.
///
/// This is the bevy [`Resource`] that our [`Item`]s will be stored in after they are loaded
#[derive(Debug, Resource, PartialEq)]
struct ItemManifest {
    items: HashMap<Id<Item>, Item>,
}

/// The raw format for [`ItemManifest`]
///
/// This is what actually gets serialized to disk when saving/loading our manifest asset.
/// Since we generate our [`Id`]s from item names, the raw storage is just a plain [`Vec`],
/// And the [`Id`]s can be generated when processing the raw manifest into the standard manifest.
#[derive(Debug, Asset, TypePath, Serialize, Deserialize, PartialEq)]
struct RawItemManifest {
    items: Vec<RawItem>,
}

impl Manifest for ItemManifest {
    // Because we're using a different format for raw/final data,
    // we need to specify both types here
    type Item = Item;
    type RawItem = RawItem;
    // Similarly, the manifest types also need to be converted
    type RawManifest = RawItemManifest;
    // Asset loading always returns a Handle, so our conversion is technically infallable.
    // Asset loading can still fail further down the pipeline, so this would have to be handled seprately.
    type ConversionError = std::convert::Infallible;

    // Our manifest uses a RON file under the hood.
    // Various common formats are supported out-of-the-box; check the [`ManifestFormat`] docs for more details
    // and remember to enable the corresponding feature in your `Cargo.toml`!
    const FORMAT: ManifestFormat = ManifestFormat::Ron;

    fn get(&self, id: Id<Item>) -> Option<&Self::Item> {
        self.items.get(&id)
    }

    // After the raw manifest is deserialied from the disk, we need to process the data slightly.
    // In this case, we need to look up and load our sprite assets, and store the handles.
    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        world: &mut World,
    ) -> Result<Self, Self::ConversionError> {
        // Asset server to load our sprite assets
        let asset_server = world.resource::<AssetServer>();

        let items: HashMap<_, _> = raw_manifest
            .items
            .into_iter()
            .map(|raw_item| {
                // Load the sprite from the path provided in the raw data
                let sprite_handle = asset_server.load(raw_item.sprite);

                // Construct actual item data
                // Most of this is identical, except for the newly generated asset handle
                let item = Item {
                    name: raw_item.name,
                    description: raw_item.description,
                    value: raw_item.value,
                    weight: raw_item.weight,
                    max_stack: raw_item.max_stack,
                    sprite: sprite_handle,
                };

                // Build an Id for our item, so it can be looked up later
                let id = Id::from_name(&item.name);

                (id, item)
            })
            .collect();

        Ok(ItemManifest { items })
    }
}

fn main() {
    App::new()
        // This example is TUI only, but the default plugins are used because they contain a bunch of asset loading stuff we need.
        .add_plugins(DefaultPlugins)
        // This is our simple state, used to navigate the asset loading process.
        .init_state::<SimpleAssetState>()
        // Coordinates asset loading and state transitions.
        .add_plugins(ManifestPlugin::<SimpleAssetState>::default())
        // Registers our item manifest, triggering it to be loaded.
        .register_manifest::<ItemManifest>("raw_items.ron")
        .add_systems(
            Update,
            list_available_items.run_if(in_state(SimpleAssetState::Ready)),
        )
        .run();
}

/// This system reads the generated item manifest resource and prints out all the items.
fn list_available_items(
    item_manifest: Res<ItemManifest>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (id, item) in item_manifest.items.iter() {
        println!("{:?}: {:?}", id, item);
    }

    // We are out of here
    app_exit_events.send_default();
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
    fn generate_raw_item_manifest() {
        let mut items = Vec::default();

        items.push(RawItem {
            name: "sword".into(),
            description: "A sharp sword".into(),
            value: 10,
            weight: 2.0,
            max_stack: 1,
            sprite: PathBuf::from("sprites/sword.png"),
        });

        items.push(RawItem {
            name: "shield".into(),
            description: "A sturdy shield".into(),
            value: 5,
            weight: 5.0,
            max_stack: 1,
            sprite: PathBuf::from("sprites/shield.png"),
        });

        let item_manifest = RawItemManifest { items };

        let serialized = ron::ser::to_string_pretty(&item_manifest, Default::default()).unwrap();
        println!("{}", serialized);

        // Save the results, to ensure that our example has a valid manifest to read.
        std::fs::write("assets/raw_items.ron", &serialized).unwrap();

        let deserialized: RawItemManifest = ron::de::from_str(&serialized).unwrap();

        assert_eq!(item_manifest, deserialized);
    }
}
