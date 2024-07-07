//! While the [SimpleAssetState`](leafwing_manifest::asset_state::SimpleAssetState) enum is a good start, it's not very flexible.
//!
//! You may want to use a more sophisticated runtime approach to loading and unloading assets, or simply integrate into an existing asset loading solution.
//! While you can use the [`AssetLoadingState`](leafwing_manifest::asset_state::AssetLoadingState) trait to define your own asset loading states, that's not the only escape hatch!
//!
//! As this example demonstrates, you can bypass the [`ManifestPlugin`](leafwing_manifest::plugin::ManifestPlugin) entirely, and load your assets however you like,
//! calling the publicly exposed methods yourself to replicate the work it does.

use std::future::Future;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext, LoadState},
    prelude::*,
    utils::ConditionalSendFuture,
};
use bevy_common_assets::ron::RonLoaderError;
use leafwing_manifest::manifest::Manifest;
use manifest_definition::{ItemManifest, RawItemManifest};

/// The core data structures and [`Manifest`] implementation is stolen directly from raw_manifest.rs:
/// it's not the focus of this example!
mod manifest_definition {
    use std::path::PathBuf;

    use bevy::{prelude::*, utils::HashMap};
    use leafwing_manifest::{
        identifier::Id,
        manifest::{Manifest, ManifestFormat},
    };
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq)]
    #[allow(dead_code)] // Properties are for demonstration purposes only.
    pub struct Item {
        name: String,
        description: String,
        value: i32,
        weight: f32,
        max_stack: u8,
        sprite: Handle<Image>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    pub struct RawItem {
        name: String,
        description: String,
        value: i32,
        weight: f32,
        max_stack: u8,
        sprite: PathBuf,
    }

    #[derive(Debug, Resource, PartialEq)]
    pub struct ItemManifest {
        items: HashMap<Id<Item>, Item>,
    }

    #[derive(Debug, Asset, TypePath, Serialize, Deserialize, PartialEq, Clone)]
    pub struct RawItemManifest {
        items: Vec<RawItem>,
    }

    impl Manifest for ItemManifest {
        type Item = Item;
        type RawItem = RawItem;
        type RawManifest = RawItemManifest;
        type ConversionError = std::convert::Infallible;

        const FORMAT: ManifestFormat = ManifestFormat::Ron;

        fn get(&self, id: Id<Item>) -> Option<&Self::Item> {
            self.items.get(&id)
        }

        fn from_raw_manifest(
            raw_manifest: Self::RawManifest,
            world: &mut World,
        ) -> Result<Self, Self::ConversionError> {
            let asset_server = world.resource::<AssetServer>();

            let items: HashMap<_, _> = raw_manifest
                .items
                .into_iter()
                .map(|raw_item| {
                    let sprite_handle = asset_server.load(raw_item.sprite);

                    let item = Item {
                        name: raw_item.name,
                        description: raw_item.description,
                        value: raw_item.value,
                        weight: raw_item.weight,
                        max_stack: raw_item.max_stack,
                        sprite: sprite_handle,
                    };

                    let id = Id::from_name(&item.name);

                    (id, item)
                })
                .collect();

            Ok(ItemManifest { items })
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Check out this system for all the fancy state management
        .add_systems(PreUpdate, manage_manifests)
        // Remember to initialize your asset types!
        .init_asset::<RawItemManifest>()
        // Remember to add the required asset loaders!
        .register_asset_loader(ItemAssetLoader)
        .run();
}

// Writing your own asset loaders is quite a bit of boilerplate:
// you need a unique asset loader for each manifest type you want to load.
// Many thanks to bevy_common_assets for showing us how to do this!
struct ItemAssetLoader;

impl AssetLoader for ItemAssetLoader {
    type Asset = RawItemManifest;
    type Settings = ();
    type Error = RonLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture
           + Future<Output = Result<<Self as AssetLoader>::Asset, <Self as AssetLoader>::Error>>
    {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let asset = ron::de::from_bytes::<RawItemManifest>(&bytes)?;
            Ok(asset)
        })
    }

    // The extensions method is ultimately used as a fallback: it isn't helpful for our workflow.
    fn extensions(&self) -> &[&str] {
        &[]
    }
}

// The same basic workflow applies when managing manifests manually, but you have more control over the process:
// 1. Start loading the raw manifest
// 2. Wait for the raw manifest to load
// 3. Convert it into a usable (non-raw manifest) form
// 4. Store it as a resource
#[derive(Debug, PartialEq, Default)]
enum ManifestProgress {
    #[default]
    NotLoaded,
    Loading,
    Loaded,
    Processed,
}

/// Handles the entire lifecycle of the manifest.
///
/// This is done here with a tiny and pretty janky single system for simplicity: the core steps can be arranged however you'd like.
/// See the source code of the [`plugin`](leafwing_manifest::plugin) module for further inspiration.
fn manage_manifests(
    mut progress: Local<ManifestProgress>,
    mut manifest_handle: Local<Option<Handle<RawItemManifest>>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    raw_manifest_assets: Res<Assets<RawItemManifest>>,
    maybe_final_manifest: Option<Res<ItemManifest>>,
) {
    match *progress {
        // Step 1: Start loading the raw manifest.
        ManifestProgress::NotLoaded => {
            // Load the raw manifest from disk
            let handle = asset_server.load("raw_items.ron");
            *manifest_handle = Some(handle);
            *progress = ManifestProgress::Loading;
        }
        // Step 2: Wait for the raw manifest to load.
        ManifestProgress::Loading => {
            // The handle is always created in the previous step, so this is safe.
            let handle = manifest_handle.as_ref().unwrap();
            // Check if the asset is loaded
            let load_state = asset_server.get_load_state(handle).unwrap();
            match load_state {
                // We're safely loaded: timie to move on to the next step.
                LoadState::Loaded => {
                    *progress = ManifestProgress::Loaded;
                }
                // We're still waiting for the asset to load
                LoadState::NotLoaded | LoadState::Loading => (),
                // Something went wrong: panic!
                LoadState::Failed(err) => {
                    panic!("Failed to load manifest: {}", err);
                }
            }
        }
        // Step 3: Process the raw manifest into a usable form.
        // Step 4: Store the usable form as a resource.
        ManifestProgress::Loaded => {
            let raw_manifest = raw_manifest_assets
                .get(manifest_handle.as_ref().unwrap())
                .unwrap()
                // This process can be done without cloning, but it involves more sophisticated machinery.
                .clone();

            // We're deferring the actual work with commands to avoid blocking the whole world
            // every time this system runs.
            commands.add(|world: &mut World| {
                let item_manifest = ItemManifest::from_raw_manifest(raw_manifest, world).unwrap();

                world.insert_resource(item_manifest);
            });
            *progress = ManifestProgress::Processed;
        }
        // Let's double check that this worked!
        ManifestProgress::Processed => {
            if let Some(final_manifest) = maybe_final_manifest {
                info_once!("Final manifest is ready: {:?}", final_manifest);
            }
        }
    }
}
