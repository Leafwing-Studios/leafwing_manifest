use std::error::Error;

use bevy::{
    asset::Asset,
    ecs::{system::Resource, world::World},
};
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
/// The elements of the manifest should generally be treated as immutable, as they are shared across the game,
/// and represent the "canonical" version of the game objects.
/// However, mutable accessors are provided, allowing for the runtime addition of new game objects,
/// as might be used for things like user-generated content or modding.
pub trait Manifest: Sized + Resource {
    /// The type of the game object stored in the manifest.
    type Item;
    /// The error type that can occur when converting raw manifests into a manifest.
    type ConversionError: Clone + PartialEq + Error;
    /// The raw data type that is loaded from disk.
    type RawManifest: Asset;
    /// The raw data type that is stored in the manifest.
    type RawItem;

    /// Converts a raw manifest into the corresponding manifest.
    ///
    /// This is an inherently fallible operation, as the raw data may be malformed or invalid.
    fn from_raw_manifest(
        raw_manifest: &Self::RawManifest,
        _world: &World,
    ) -> Result<Self, Self::ConversionError>;

    /// Converts a raw item into the corresponding item.
    ///
    /// This is an inherently fallible operation, as the raw data may be malformed or invalid.
    fn convert_raw_item(raw_item: &Self::RawItem) -> Result<Self::Item, Self::ConversionError>;

    /// Converts and then inserts a raw item into the manifest.
    ///
    /// This is a convenience method that combines the conversion and insertion steps.
    fn insert_raw_item(
        &mut self,
        raw_item: &Self::RawItem,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>> {
        Self::convert_raw_item(raw_item)
            .map_err(|e| ManifestModificationError::ConversionFailed(e))
            .and_then(|item| self.insert(item))
    }

    /// Inserts a new item into the manifest.
    ///
    /// The item is given a unique identifier, which is returned.
    fn insert(
        &mut self,
        item: Self::Item,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>>;

    /// Removes an item from the manifest.
    ///
    /// The item removed is returned, if it was found.
    fn remove(
        &mut self,
        id: &Id<Self::Item>,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>>;

    /// Gets an item from the manifest by its unique identifier.
    ///
    /// Returns [`None`] if no item with the given ID is found.
    fn get(&self, id: &Id<Self::Item>) -> Option<&Self::Item>;

    /// Gets a mutable reference to an item from the manifest by its unique identifier.
    ///
    /// Returns [`None`] if no item with the given ID is found.
    fn get_mut(&mut self, id: &Id<Self::Item>) -> Option<&mut Self::Item>;
}

/// A trait for manifests that have named items.
///
/// Naming items can be useful for quick-prototyping, or for hybrid code and data-driven workflows.
///
/// However, named items can be less efficient than using [`Id`]s, as they require string lookups and an additional string-based mapping.
/// As a result, the methods of this trait have been split from the main [`Manifest`] trait,
/// and should be used with deliberation.
pub trait NamedManifest: Manifest {
    /// Gets the unique identifier of an item by its name.
    ///
    /// Returns [`None`] if no item with the given name is found.
    fn id_of(&self, name: &str) -> Option<Id<Self::Item>>;

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

    /// Gets an item from the manifest by its name.
    ///
    /// Returns [`None`] if no item with the given name is found.
    fn get_by_name(&self, name: &str) -> Option<&Self::Item> {
        self.id_of(name).and_then(|id| self.get(&id))
    }

    /// Gets a mutable reference to an item from the manifest by its name.
    ///
    /// Returns [`None`] if no item with the given name is found.
    fn get_mut_by_name(&mut self, name: &str) -> Option<&mut Self::Item> {
        self.id_of(name).and_then(move |id| self.get_mut(&id))
    }
}

/// An error that can occur when modifying a manifest.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ManifestModificationError<M: Manifest> {
    #[error("The name {} is already in use.", _0)]
    DuplicateName(String),
    #[error("The raw item could not be converted.")]
    ConversionFailed(M::ConversionError),
    #[error("The item with ID {} was not found.", _0)]
    NotFound(u64),
    #[error("No item with the name {} was found.", _0)]
    NameNotFound(String),
}
