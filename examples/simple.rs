use bevy::{prelude::*, utils::HashMap};
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    identifier::Id,
    manifest::{Manifest, ManifestModificationError},
    plugin::{AppExt, ManifestPlugin},
};

/// The data for as single [`ItemType`].
///
/// This is the data that is shared between all items of the same type.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Item {
    name: String,
    description: String,
    value: i32,
    weight: f32,
    max_stack: i32,
}

/// A data-driven manifest, which contains all the data for all the items in the game.
// FIXME: remove Clone requirement
#[derive(Debug, Resource, Asset, TypePath, Clone)]
struct ItemManifest {
    items: HashMap<Id<Item>, Item>,
}

impl Manifest for ItemManifest {
    type Item = Item;
    type RawItem = Item;
    type RawManifest = ItemManifest;
    type ConversionError = std::convert::Infallible;

    fn get(&self, id: &Id<Item>) -> Option<&Self::Item> {
        self.items.get(id)
    }

    fn get_mut(&mut self, id: &Id<Item>) -> Option<&mut Self::Item> {
        self.items.get_mut(id)
    }

    fn insert(
        &mut self,
        item: Self::Item,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>> {
        // Names can be used as unique identifiers for items;
        // the from_name method quickly hashes the string into a unique ID.
        let id = Id::from_name(item.name.clone());

        // Because we're relying on the name as a unique identifier,
        // we need to check for duplicates.
        if self.items.contains_key(&id) {
            return Err(ManifestModificationError::DuplicateName(item.name.clone()));
        } else {
            self.items.insert(id, item);
            Ok(id)
        }
    }

    fn remove(
        &mut self,
        id: &Id<Self::Item>,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>> {
        self.items.remove(id);
        Ok(*id)
    }

    // We're able to read the data directly from the serialized format,
    // so there's no need for any intermediate conversion.
    fn from_raw_manifest(
        raw_manifest: &Self::RawManifest,
        _world: &mut World,
    ) -> Result<Self, Self::ConversionError> {
        Ok(raw_manifest.clone())
    }

    fn convert_raw_item(raw_item: &Self::RawItem) -> Result<Self::Item, Self::ConversionError> {
        Ok(raw_item.clone())
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
        .register_manifest::<ItemManifest>("assets/items.ron".into())
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
