//! This example demonstrates how to "preload" assets before processing manifests. This might be done through
//! a third-party library such as bevy_asset_loader, or manually as shown below.
//!
//! This example is very similar to the raw_manifest.rs example, however separates the texture and manifest loading
//! into separate states and uses the preloaded assets in `from_raw_manifest`.

use bevy::{app::AppExit, log::LogPlugin, prelude::*, state::app::StatesPlugin};
use leafwing_manifest::{
    asset_state::AssetLoadingState,
    plugin::{ManifestPlugin, RegisterManifest},
};

/// This is very similar to the `raw_manifest` example. The main difference
/// is in the `from_raw_manifest` implementation below which uses [LoadedAssets]
/// to retrieve asset handles.
pub mod manifest_definition {
    use bevy::{prelude::*, utils::HashMap};
    use leafwing_manifest::{
        identifier::Id,
        manifest::{Manifest, ManifestFormat},
    };
    use serde::{Deserialize, Serialize};

    use crate::LoadedAssets;

    /// The data for as single item that might be held in the player's inventory.
    ///
    /// This is the format that our item data is stored in after it's been loaded into a Bevy [`Resource`].
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

    /// The raw format for [`Item`] data.
    ///
    /// This is used inside of [`RawItemManifest`] to be saved/loaded to disk as an [`Asset`].
    /// The only difference in this case is that the `sprite` field has been changed from a loaded [`Handle<Image>`] to a [`PathBuf`].
    /// This [`PathBuf`] references the actual sprite path in our assets folder,
    /// but other identifiers could be used for more complex asset loading strategies.
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct RawItem {
        name: String,
        description: String,
        value: i32,
        weight: f32,
        max_stack: u8,
        sprite: String,
    }

    /// A data-driven manifest, which contains the canonical data for all the items in the game.
    ///
    /// This is the bevy [`Resource`] that our [`Item`]s will be stored in after they are loaded
    #[derive(Debug, Resource, PartialEq)]
    pub struct ItemManifest {
        pub items: HashMap<Id<Item>, Item>,
    }

    /// The raw format for [`ItemManifest`]
    ///
    /// This is what actually gets serialized to disk when saving/loading our manifest asset.
    /// Since we generate our [`Id`]s from item names, the raw storage is just a plain [`Vec`],
    /// And the [`Id`]s can be generated when processing the raw manifest into the standard manifest.
    #[derive(Debug, Asset, TypePath, Serialize, Deserialize, PartialEq)]
    pub struct RawItemManifest {
        items: Vec<RawItem>,
    }

    impl Manifest for ItemManifest {
        // Because we're using a different format for raw/final data,
        // we need to specify both types here
        type Item = Item;
        type RawItem = RawItem;
        // Similarly, the manifest types also need to be converted
        type RawManifest = RawItemManifest;
        // Asset loading always returns a Handle, so our conversion is technically infallible.
        // Asset loading can still fail further down the pipeline, which would have to be handled separately.
        type ConversionError = std::convert::Infallible;

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
            let assets = world.resource::<LoadedAssets>();

            let items: HashMap<_, _> = raw_manifest
                .items
                .into_iter()
                .map(|raw_item| {
                    // Load the sprite from the path provided in the raw data
                    let sprite_handle = match raw_item.sprite.as_str() {
                        "sprites/sword.png" => assets.sword.clone(),
                        "sprites/shield.png" => assets.shield.clone(),
                        _ => panic!("Unknown asset - {}", raw_item.sprite),
                    };

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
}

/// We create a GameState, where only some of the states are relevant to leafwing_manifest.
#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, States)]
enum GameState {
    #[default]
    PreloadAssets,
    LoadManifests,
    ProcessManifests,
    ManifestError,
    Ready,
}

/// Make sure leafwing_manifest knows which states are relevant for manifest processing.
impl AssetLoadingState for GameState {
    const LOADING: Self = Self::LoadManifests;
    const PROCESSING: Self = Self::ProcessManifests;
    const READY: Self = Self::Ready;
    const FAILED: Self = Self::ManifestError;
}

/// We're storing some asset handles here. This could use something like bevy_asset_loader,
/// or some other custom asset pipeline.
#[derive(Resource)]
pub struct LoadedAssets {
    pub sword: Handle<Image>,
    pub shield: Handle<Image>,
}

fn main() {
    App::new()
        // leafwing_manifest requires `AssetPlugin` to function
        // This is included in `DefaultPlugins`, but this example is very small, so it only uses the `MinimalPlugins`
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            LogPlugin::default(),
            StatesPlugin,
            ImagePlugin::default(),
        ))
        // This is our simple state, used to navigate the asset loading process.
        .init_state::<GameState>()
        // Coordinates asset loading and state transitions. Note the use of `without_initial_state`
        // so that `GameState::PreloadAssets` is the initial app state, rather than
        // `AssetLoadingState::LOADING`.
        .add_plugins(ManifestPlugin::<GameState>::new(false))
        // Registers our item manifest, triggering it to be loaded when the app transitions
        // to `GameState::LoadManifests`.
        .register_manifest::<manifest_definition::ItemManifest>("raw_items.ron")
        .add_systems(OnEnter(GameState::PreloadAssets), preload_assets)
        .add_systems(OnEnter(GameState::Ready), list_available_items)
        .run();
}

fn preload_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let sword: Handle<Image> = asset_server.load("sprites/sword.png");
    let shield: Handle<Image> = asset_server.load("sprites/shield.png");
    commands.insert_resource(LoadedAssets { sword, shield });

    // in practice we could wait until its loaded here, but for simplicity we'll
    // just jump straight to manifest parsing.
    next_state.set(GameState::ProcessManifests);
}

/// This system reads the generated item manifest resource and prints out all the items.
fn list_available_items(
    item_manifest: Res<manifest_definition::ItemManifest>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (id, item) in item_manifest.items.iter() {
        info!("{:?}: {:?}", id, item);
    }

    // We are out of here
    app_exit_events.send_default();
}
