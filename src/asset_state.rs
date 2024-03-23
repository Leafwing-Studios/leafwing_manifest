use bevy::ecs::schedule::States;

/// A trait that translates your custom [`States`] enum into the states required for asset loading.
///
/// Note that you are not required to use this trait.
/// Instead, you can add or emulate the required systems from [`ManifestPlugin`](crate::plugin::ManifestPlugin) manually to match your app logic.
pub trait AssetLoadingState: States {
    /// Assets are currently being loaded.
    const LOADING: Self;
    /// Assets have been loaded successfully,
    /// but are not yet ready to be used.
    const PROCESSING: Self;
    /// Assets are ready to be used.
    const READY: Self;
    /// Assets failed to load.
    ///
    /// Check the logs for more information.
    const FAILED: Self;
}

/// A simple [`States`] enum for asset loading.
///
/// This pattern is very simple, and suited for small applications that can afford to load all assets at once.
#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy, Default, States)]
pub enum SimpleAssetState {
    /// Assets are currently being loaded.
    #[default]
    Loading,
    /// Assets have been loaded successfully,
    /// but are not yet ready to be used.
    Processing,
    /// Assets are ready to be used.
    Ready,
    /// Assets failed to load.
    ///
    /// Check the logs for more information.
    Failed,
}

impl AssetLoadingState for SimpleAssetState {
    const LOADING: Self = SimpleAssetState::Loading;
    const PROCESSING: Self = SimpleAssetState::Processing;
    const READY: Self = SimpleAssetState::Ready;
    const FAILED: Self = SimpleAssetState::Failed;
}
