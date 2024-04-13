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
fn main() {}
