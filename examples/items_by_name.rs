//! When generating content through a combination of code and assets,
//! it is often useful to be able to quickly spawn new instances of objects by name from inside the code.
//!
//! This workflow tends to make the most sense during prototyping, where the exact details of the content are still in flux,
//! or when the content is generated procedurally and the exact details are not known ahead of time.
//! By contrast, traditional "levels" generally make most sense as pure assets,
//! although even then it can be useful to be able to spawn objects by name for scripting purposes:
//! creating effects, dynamically spawning enemies, etc.
//!
//! This example shows how to use manifests to create a simple system for working with manifest entries by name,
//! although the same principles can be used to manipulate manifests if using the [`MutableManifest`] trait as well.
//!
//! This code is largely copied from the `simple.rs` example: we're just adding constants and a new system to demonstrate the name-based lookups.

use bevy::{log::LogPlugin, prelude::*, utils::HashMap};
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
    plugin::{ManifestPlugin, RegisterManifest},
};
use serde::{Deserialize, Serialize};

// While you *can* simply use the name directly via the various name-based methods on the Manifest trait,
// it's generally a good idea to store constants for the names you're going to use when possible.
// This has three main benefits:
// 1. It makes it easier to refactor your code later, as you can change the name in one place and have it propagate everywhere.
// 2. It makes it easier to catch typos, as the compiler will catch any references to a name that doesn't exist.
// 3. It saves on recomputing the hash of the name every time you need it.
const SWORD: Id<Item> = Id::from_name("sword");
const SHIELD: Id<Item> = Id::from_name("shield");

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)] // Properties are for demonstration purposes only.
struct Item {
    name: String,
    description: String,
    value: i32,
    weight: f32,
    max_stack: u8,
}

#[derive(Debug, Resource, Asset, TypePath, Serialize, Deserialize, PartialEq)]
struct ItemManifest {
    items: HashMap<Id<Item>, Item>,
}

impl Manifest for ItemManifest {
    type Item = Item;
    type RawItem = Item;
    type RawManifest = ItemManifest;
    type ConversionError = std::convert::Infallible;

    const FORMAT: ManifestFormat = ManifestFormat::Ron;

    fn get(&self, id: Id<Item>) -> Option<&Self::Item> {
        self.items.get(&id)
    }

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        _world: &mut World,
    ) -> Result<Self, Self::ConversionError> {
        Ok(raw_manifest)
    }
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, AssetPlugin::default(), LogPlugin::default()))
        .init_state::<SimpleAssetState>()
        .add_plugins(ManifestPlugin::<SimpleAssetState>::default())
        .register_manifest::<ItemManifest>("items.ron")
        .add_systems(OnEnter(SimpleAssetState::Ready), look_up_items_by_name)
        .run();
}

/// This system reads the generated item manifest resource and prints out all the items.
fn look_up_items_by_name(item_manifest: Res<ItemManifest>) {
    // Look up the items by name.
    let sword = item_manifest.get(SWORD);
    let shield = item_manifest.get(SHIELD);

    // Print out the items.
    if let Some(sword) = sword {
        println!("Found sword: {:?}", sword);
    } else {
        println!("Sword not found!");
    }

    if let Some(shield) = shield {
        println!("Found shield: {:?}", shield);
    } else {
        println!("Shield not found!");
    }

    // We could also use the `get_by_name` method, which is a bit more concise,
    // but doesn't provide the same level of type safety as using the `Id` directly.
    // However, using these methods is the right choice when working with truly dynamic inputs:
    // for example, when reading from a file or user input.
    let sword = item_manifest.get_by_name("sword");
    let shield = item_manifest.get_by_name("shield");

    if let Some(sword) = sword {
        println!("Found sword by name: {:?}", sword);
    } else {
        println!("Sword not found by name!");
    }

    if let Some(shield) = shield {
        println!("Found shield by name: {:?}", shield);
    } else {
        println!("Shield not found by name!");
    }
}
