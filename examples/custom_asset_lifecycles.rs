//! While the [SimpleAssetState`](leafwing_manifest::asset_state::SimpleAssetState) enum is a good start, it's not very flexible.
//!
//! You may want to use a more sophisticated runtime approach to loading and unloading assets, or simply integrate into an existing asset loading solution.
//! While you can use the [`AssetLoadingState`](leafwing_manifest::asset_state::AssetLoadingState) trait to define your own asset loading states, that's not the only escape hatch!
//!
//! As this example demonstrates, you can bypass the [`ManifestPlugin`](leafwing_manifest::plugin::ManifestPlugin) entirely, and load your assets however you like,
//! calling the publicly exposed methods yourself to replicate the work it does.

fn main() {}
