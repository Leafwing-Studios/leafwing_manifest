use std::path::PathBuf;

use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetLoadFailedEvent, AssetServer, LoadState, UntypedHandle};
use bevy::ecs::prelude::*;
use bevy::log::{error, info};
use bevy::utils::{HashMap, HashSet};

use crate::asset_state::AssetLoadingState;
use crate::manifest::Manifest;

/// A plugin for loading assets from a [`Manifest`].
///
/// This plugin will add the required state to your app (starting in [`AppLoadingState::LOADING`]),
/// and set up the required systems to progress through the asset loading process and parse any added manifests.
///
/// Note that manifests must be added to the app manually, using the [`app.register_manifest`](crate::AppExt::register_manifest) method.
/// This plugin **must** be added before manifests are registered.
///
/// This plugin is intenionally optional: if you have more complex asset loading requirements, take a look at the systems in this plugin and either add or reimplement them as needed.
pub struct ManifestPlugin<S: States> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: AssetLoadingState> Plugin for ManifestPlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_state(S::LOADING)
            .init_resource::<AssetTracker>()
            .add_systems(
                Update,
                (start_loading_assets, check_if_assets_have_loaded::<S>)
                    .chain()
                    .run_if(in_state(S::LOADING)),
            );
    }
}

/// An extension trait for registering manifests with an app.
pub trait AppExt {
    /// Registers a manifest with the app, preparing it for loading and parsing.
    ///
    /// The final manifest type must implement [`Manifest`], while the raw manifest type must implement [`Asset`](bevy::asset::Asset).
    fn register_manifest<M: Manifest>(&mut self, path: PathBuf);
}

impl AppExt for App {
    fn register_manifest<M: Manifest>(&mut self, path: PathBuf) {
        let mut asset_tracker = self.world.resource_mut::<AssetTracker>();
        asset_tracker.register(path);
        self.add_systems(
            Update,
            report_failed_raw_manifest_loading::<M>
                .run_if(on_event::<AssetLoadFailedEvent<M::RawManifest>>()),
        );
    }
}

/// Keeps track of the assets that need to be loaded, and their loading progress.
///
/// You can add your own assets to this tracker using [`AssetTracker::register`],
/// which can be useful to ensure that the [`AssetLoadingState`] state declared in [`ManifestPlugin`]
/// only advances when all needed assets have been loaded.
#[derive(Resource, Debug, Default)]
pub struct AssetTracker {
    assets_to_load: HashSet<PathBuf>,
    initialized_assets: HashMap<PathBuf, UntypedHandle>,
    asset_progress: HashMap<PathBuf, LoadState>,
}

impl AssetTracker {
    /// Registers an asset to be loaded.
    ///
    /// This must be done before [`AssetLoadingState::LOADING`] is complete.
    pub fn register(&mut self, path: PathBuf) {
        self.assets_to_load.insert(path);
    }

    /// Returns a reference to the set of assets that need to be loaded.
    pub fn assets_to_load(&self) -> &HashSet<PathBuf> {
        &self.assets_to_load
    }

    /// Returns a reference to the assets that have been initialized but not yet loaded.
    pub fn initialized_assets(&self) -> &HashMap<PathBuf, UntypedHandle> {
        &self.initialized_assets
    }

    /// Returns a reference to the recorded loading progress of assets.
    pub fn asset_progress(&self) -> &HashMap<PathBuf, LoadState> {
        &self.asset_progress
    }
}

/// Queues up assets to be loaded asynchronously.
pub fn start_loading_assets(
    mut asset_tracker: ResMut<AssetTracker>,
    asset_server: Res<AssetServer>,
) {
    let assets_to_load = std::mem::take(&mut asset_tracker.assets_to_load);

    for path in assets_to_load {
        let handle = asset_server.load_untyped(path.clone()).untyped();
        asset_tracker.initialized_assets.insert(path, handle);
    }
}

/// Checks if all registered assets have loaded,
/// and progresses to the next state if they have.
pub fn check_if_assets_have_loaded<S: AssetLoadingState>(
    asset_server: Res<AssetServer>,
    mut asset_tracker: ResMut<AssetTracker>,
    mut next_state: ResMut<NextState<S>>,
) {
    let mut assets_loaded = true;

    for (path, handle) in &asset_tracker.initialized_assets.clone() {
        let load_state = asset_server
            .get_load_state(handle)
            .unwrap_or(LoadState::Failed);

        match load_state {
            LoadState::Loaded => {
                asset_tracker.assets_to_load.remove(path);
                info!("Loaded asset: {:?}", path);
            }
            LoadState::Failed => {
                error!("Failed to load asset: {:?}", path);
            }
            _ => {}
        }

        if load_state == LoadState::Failed {
            error!("Failed to load asset: {:?}", path);
            continue;
        }

        asset_tracker
            .asset_progress
            .insert(path.clone(), load_state);

        if load_state != LoadState::Loaded {
            assets_loaded = false;
        }
    }

    if assets_loaded {
        asset_tracker.initialized_assets.clear();
        asset_tracker.assets_to_load.clear();

        next_state.set(S::PROCESSING);
    }
}

/// Watches for and reports failed raw manifest loading events.
///
/// This generic system is currently required as [`LoadState::Failed`] does not contain the error that caused the failure.
///
/// See [bevy#12667](https://github.com/bevyengine/bevy/issues/12667) for more information.0
pub fn report_failed_raw_manifest_loading<M: Manifest>(
    mut events: EventReader<AssetLoadFailedEvent<M::RawManifest>>,
) {
    for event in events.read() {
        error!(
            "Failed to load asset at {} due to {:?}",
            event.path, event.error
        );
    }
}

/// A system which processes a raw manifest into a completed [`Manifest`],
/// and then stores the manifest as a [`Resource`] in the [`World`].
pub fn process_manifest<M: Manifest>(
    raw_manifest: &M::RawManifest,
    world: &mut World,
) -> Result<(), M::ConversionError> {
    match M::from_raw_manifest(&raw_manifest) {
        Ok(manifest) => {
            world.insert_resource(manifest);
            Ok(())
        }
        Err(err) => {
            error!("Failed to process manifest: {:?}", err);
            Err(err)
        }
    }
}
