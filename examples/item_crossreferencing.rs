//! While manifests sometimes rely on other manifests, it's also possible for items within a manifest to reference each other directly!
//!
//! In this example, we have a collection of species, each of which has a list of viable prey.
//! We can use the [`Id`] of each species to reference other species in the manifest,
//! and perform validation to ensure that the prey species have their own valid entries in the final manifest.

fn main() {}
