use std::error::Error;

use bevy::{
    asset::Asset,
    ecs::{system::Resource, world::World},
};
use serde::Deserialize;
use thiserror::Error;

use crate::identifier::Id;

/// A manifest is a collection of ready-to-use game objects,
/// which are loaded from disk and stored in the ECS as a resource.
///
/// The data on the disk is stored in a serialization-friendly format: [`Manifest::RawManifest`].
/// These types have simple structures and are easy to read and write.
/// Once these are all loaded, they are processed into the final manifest.
///
/// With a manifest in hand, game objects are looked up by their unique [`Id`],
/// returning an object of type [`Manifest::Item`].
///
/// Types that implement [`Manifest`] are generally simple hashmap data structures, mapping `Id<Item>` to `Item`.
/// However, if looking up objects by their name is required, the [`NamedManifest`] trait can be added,
/// and strings are used as the key instead of `Id<Item>`.
/// The `Id` can then be quickly generated, using the built-in [`Id::from_name`] stable hash method.
///
/// The elements of the manifest should generally be treated as immutable, as they are shared across the game,
/// and represent the "canonical" version of the game objects.
/// However, mutable accessors are provided under the [`MutableManifest`] trait, allowing for the runtime addition of new game objects,
/// as might be used for things like user-generated content, manifest-creation tools or modding.
pub trait Manifest: Sized + Resource {
    /// The raw data type that is loaded from disk.
    ///
    /// This type may be `Self`, if no further processing is required.
    ///
    /// While the raw manifest *can* be stored on disk as a dictionary/map of items,
    /// keyed by either their name or `Id`, it is generally more efficient (and easier to hand-author)
    /// if it is instead stored as a simple flat list.
    type RawManifest: Asset + for<'de> Deserialize<'de>;

    /// The raw data type that is stored in the manifest.
    type RawItem;

    /// The type of the game object stored in the manifest.
    ///
    /// These are commonly [`Bundle`](bevy::ecs::bundle::Bundle) types, allowing you to directly spawn them into the [`World`].
    /// If you wish to store [`Handles`](bevy::asset::Handle) to other assets (such as textures, sprites or sounds),
    /// starting the asset loading process for those assets in [`from_raw_manifest`](Manifest::from_raw_manifest) works very well!
    type Item: TryFrom<Self::RawItem, Error = Self::ConversionError>;

    /// The error type that can occur when converting raw manifests into a manifest.
    ///
    /// When implementing this trait for a manifest without any conversion steps,
    /// this type can be set to [`Infallible`](std::convert::Infallible).
    ///
    /// If you want to reprocess the manifest,
    /// consider returning the raw manifest in the error type.
    type ConversionError: Error;

    /// The format of the raw manifest on disk.
    /// This is used to construct an asset loader, with the help of [`bevy_common_assets`].
    ///
    /// Several common options are available, including RON, JSON, XML and CSV.
    /// If you wish to use a custom format, you will want to set this to [`ManifestFormat::Custom`]
    /// and add your own [`bevy::asset::AssetLoader`] directly to your Bevy app.
    const FORMAT: ManifestFormat;

    /// Converts a raw manifest into the corresponding manifest.
    ///
    /// This is an inherently fallible operation, as the raw data may be malformed or invalid.
    ///
    /// If you wish to reference assets in the [`Item`](Manifest::Item) type, you can start the asset loading process here,
    /// and store a strong reference to the [`Handle`](bevy::asset::Handle) in the item.
    ///
    /// If you need access to data from *other* manifests, you can use the [`World`] to look them up as resources.
    /// This is useful for cross-referencing data between manifests.
    /// Use ordinary system ordering to ensure that the required manifests are loaded first:
    /// the system that calls this method is [`process_manifest::<M>`](crate::plugin::process_manifest), run in the [`PreUpdate`](bevy::prelude::PreUpdate) schedule.
    ///
    /// This method is commonly implemented using the [`TryFrom`] trait between [`Self::RawItem`](Manifest::RawItem) and [`Self::Item`](Manifest::Item).
    /// By iterating over the items in the raw manifest, you can convert them into the final item type one at a time.
    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        world: &mut World,
    ) -> Result<Self, Self::ConversionError>;

    /// Gets an item from the manifest by its unique identifier.
    ///
    /// Returns [`None`] if no item with the given ID is found.
    fn get(&self, id: &Id<Self::Item>) -> Option<&Self::Item>;
}

/// The file format of the raw manifest on disk.
///
/// All of the corresponding features are off by default, and must be enabled with feature flags.
/// Check the `Cargo.toml` file for the list of available features.
pub enum ManifestFormat {
    #[cfg(feature = "ron")]
    /// A Rust-specific configuration format that is easy for both humans and machines to read and write.
    Ron,
    #[cfg(feature = "json")]
    /// A standard configuration format that is easy for both humans and machines to read and write.
    Json,
    #[cfg(feature = "yaml")]
    /// A configuration format that accepts complex data structures, with a focus on human-editable data.
    Yaml,
    #[cfg(feature = "toml")]
    /// A configuration format that emphasizes readability and simplicity, with a focus on human-editable data.
    Toml,
    #[cfg(feature = "xml")]
    /// A markup language that defines a set of rules for encoding documents in a format that is both human-readable and machine-readable.
    Xml,
    #[cfg(feature = "csv")]
    /// A simple text-based tabular format, with rows separated by newlines and columns separated by commas.
    Csv,
    #[cfg(feature = "msgpack")]
    /// A JSON-derived binary format.
    MsgPack,
    /// Your own custom format.
    ///
    /// If this is selected, you will need to create and register your own [`bevy::asset::AssetLoader`] trait for the [`Manifest::RawManifest`] asset type.
    Custom,
}

/// A trait for manifests that have named items.
///
/// Naming items can be useful for quick-prototyping, or for hybrid code and data-driven workflows.
///
/// However, named items can be less efficient than using [`Id`]s, as they require string lookups and an additional string-based mapping.
/// As a result, the methods of this trait have been split from the main [`Manifest`] trait,
/// and should be used with deliberation.
///
/// This trait can be combined with [`MutableManifest`] in the form of the [`NamedMutableManifest`] trait to allow for modification by name.
pub trait NamedManifest: Manifest {
    /// Gets the unique identifier of an item by its name.
    ///
    /// Returns [`None`] if no item with the given name is found.
    fn id_of(&self, name: &str) -> Option<Id<Self::Item>>;

    /// Gets an item from the manifest by its name.
    ///
    /// Returns [`None`] if no item with the given name is found.
    fn get_by_name(&self, name: &str) -> Option<&Self::Item> {
        self.id_of(name).and_then(|id| self.get(&id))
    }
}

/// A trait for manifests that can be modified.
///
/// In many cases, manifests are read-only, and are loaded from disk at the start of the game.
/// Mutating the data in a manifest is generally not recommended, as it can lead to inconsistencies and bugs.
/// For example, you may accidentally remove an item that is referenced elsewhere in the game,
/// or change the properties of an item that is already in use without updating all corresponding instances.
///
/// However, there are some cases where mutable manifests are useful:
/// - User-generated content, where players can create new items or modify existing ones.
/// - Modding, where the game's data can be changed to create new experiences.
/// - Debugging, where you want to quickly add or remove items to test new features.
/// - Procedural generation, where you want to create new items on the fly.
/// - Temporary changes, such as changing the properties of an item for a single level.
/// - Huge datasets, where you want to load only a subset of the data into memory at a time.
///
/// In many of these cases, only implementing this trait when a feature flag is enabled is a good way to prevent accidental modification.
///
/// This trait can be combined with [`NamedManifest`] in the form of the [`NamedMutableManifest`] trait to allow for modification by name.
pub trait MutableManifest: Manifest {
    /// Inserts a new item into the manifest.
    ///
    /// The item is given a unique identifier, which is returned.
    ///
    /// The [`Id`] typically used as a key here should be generated via the [`Id::from_name`] method,
    /// which hashes the name (fetched from a field on the raw item) into a collision-resistant identifier.
    ///
    /// If a duplicate entry is found, you should return [`Err(ManifestModificationError::DuplicateName(name))`](ManifestModificationError::DuplicateName).
    fn insert(
        &mut self,
        item: Self::Item,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>>;

    /// Converts and then inserts a raw item into the manifest.
    ///
    /// This is a convenience method that combines the conversion and insertion steps.
    fn insert_raw_item(
        &mut self,
        raw_item: Self::RawItem,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>> {
        let conversion_result = TryFrom::<Self::RawItem>::try_from(raw_item);

        match conversion_result {
            Ok(item) => self.insert(item),
            Err(e) => Err(ManifestModificationError::ConversionFailed(e)),
        }
    }

    /// Removes an item from the manifest.
    ///
    /// The item removed is returned, if it was found.
    fn remove(
        &mut self,
        id: &Id<Self::Item>,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>>;

    /// Gets a mutable reference to an item from the manifest by its unique identifier.
    ///
    /// Returns [`None`] if no item with the given ID is found.
    fn get_mut(&mut self, id: &Id<Self::Item>) -> Option<&mut Self::Item>;
}

/// A trait for manifests that have named items and can be modified.
///
/// This trait combines the [`NamedManifest`] and [`MutableManifest`] traits, and adds convenience methods for the intersection of their features.
pub trait NamedMutableManifest: NamedManifest + MutableManifest {
    /// Gets a mutable reference to an item from the manifest by its name.
    ///
    /// Returns [`None`] if no item with the given name is found.
    fn get_mut_by_name(&mut self, name: &str) -> Option<&mut Self::Item> {
        self.id_of(name).and_then(move |id| self.get_mut(&id))
    }

    /// Inserts a new item into the manifest by name.
    ///
    /// The item is given a unique identifier, which is returned.
    fn insert_by_name(
        &mut self,
        name: &str,
        item: Self::Item,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>> {
        let id = Id::from_name(name.to_string());

        if self.get(&id).is_some() {
            Err(ManifestModificationError::DuplicateName(name.to_string()))
        } else {
            self.insert(item)
        }
    }

    /// Converts and then inserts a raw item into the manifest by name.
    ///
    /// This is a convenience method that combines the conversion and insertion steps.
    fn insert_raw_item_by_name(
        &mut self,
        name: &str,
        raw_item: Self::RawItem,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>> {
        let conversion_result = TryFrom::<Self::RawItem>::try_from(raw_item);

        match conversion_result {
            Ok(item) => self.insert_by_name(name, item),
            Err(e) => Err(ManifestModificationError::ConversionFailed(e)),
        }
    }

    /// Removes an item from the manifest by name.
    ///
    /// The item removed is returned, if it was found.
    fn remove_by_name(
        &mut self,
        name: &str,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>> {
        self.id_of(name)
            .ok_or_else(|| ManifestModificationError::NameNotFound(name.to_string()))
            .and_then(|id| self.remove(&id))
    }
}

/// An error that can occur when modifying a manifest.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ManifestModificationError<M: Manifest> {
    /// The name of the item is already in use.
    #[error("The name {} is already in use.", _0)]
    DuplicateName(String),
    /// The raw item could not be converted.
    ///
    /// The error that occurred during the conversion is included.
    #[error("The raw item could not be converted.")]
    ConversionFailed(M::ConversionError),
    /// The item with the given ID was not found.
    #[error("The item with ID {:?} was not found.", _0)]
    NotFound(Id<M::Item>),
    /// The item with the given name was not found.
    #[error("No item with the name {} was found.", _0)]
    NameNotFound(String),
}
