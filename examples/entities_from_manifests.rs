//! Spawning entities based on the contents of a manifest resource is one of the most common use cases for `leafwing_manifest`.
//!
//! These might be monsters, levels, or any other kind of game object.
//!
//! There are three main patterns, each of which are showcased in this example:
//!
//! 1. Item-as-bundle: Each item in the manifest is a complete bundle of components, which are added to a single entity when it is spawned.
//! 2. Item-as-partial-bundle: Each item in the manifest contains the configurable elements of an entity's bundle, from which the final bundle is constructed.
//! 3. Item-as-scene: Each item in the manifest is a scene containing a hierarchy of entities.
//!
//!
//! The item-as-bundle pattern is the simplest, and is suitable for cases where you don't have much duplicated data between items.
//! The item-as-partial-bundle pattern is more flexible, and is suitable for cases where you have a lot of duplicated data between items that you don't want to bloat your manifest with.
//! The item-as-scene pattern is the most complex, and is suitable for cases (such as 3D models) where you actually want to spawn an entire entity hierarchy.

fn main() {}
