//! `leafwing_manifest` uses a generic identifier type, `Id<T>`, as the key to track data-driven assets throughout their lifecycle.
//!
//! This can be constructed from a string-based identifier, stored in the human-readable files,
//! that marks entries as e.g. "grass" or "hammer".

use bevy::{prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

/// The unique identifier of type `T`.
///
/// These are constructed by hashing object names via [`Id::from_name`],
/// and represent an identifier for a *kind* of object, not a unique instance of them.
///
/// [`Id`] is a tiny [`Copy`] type, used to quickly and uniquely identify game objects.
/// Unlike enum variants, these can be read from disk and constructed at runtime.
///
/// It can be stored as a component to identify the variety of game object used.
#[derive(Component, Reflect, Serialize, Deserialize)]
pub struct Id<T> {
    /// The unique identifier.
    ///
    /// This is usually the hash of a string identifier used in the manifest files.
    /// The number value is used to handle the data more efficiently in the game.
    value: u64,

    /// Marker to make the compiler happy
    #[reflect(ignore)]
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

/// A constant used in the hashing algorithm of the IDs.
///
/// This should be a positive prime number, roughly equal to the number of characters in the input alphabet.
const HASH_P: u64 = 53;

/// A constant used in the hashing algorithm of the IDs.
///
/// This should be a large prime number as it is used for modulo operations.
/// Larger numbers have a lower chance of a hash collision.
const HASH_M: u64 = 1_000_000_009;

impl<T> Id<T> {
    /// Creates a new ID from human-readable string identifier.
    ///
    /// The ID is created as a non-invertible hash of the string.
    // TODO: make this a const fn. Feel free to change the hashing algorithm as needed.
    // Tracked in https://github.com/Leafwing-Studios/leafwing_manifest/issues/19
    pub fn from_name(name: &str) -> Self {
        // Algorithm adopted from <https://cp-algorithms.com/string/string-hashing.html>

        let mut value = 0;
        let mut p_pow = 1;

        for &byte in name.as_bytes() {
            value = (value + (byte as u64 + 1) * p_pow) % HASH_M;
            p_pow = (p_pow * HASH_P) % HASH_M;
        }

        Id {
            value,
            _phantom: PhantomData::default(),
        }
    }
}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Id").field("value", &self.value).finish()
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}
