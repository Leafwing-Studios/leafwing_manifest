use std::error::Error;

use bevy::{
    asset::Asset,
    ecs::{system::Resource, world::World},
};

pub trait Manifest: Sized + Resource {
    type Item;
    type Err: Error;
    type RawManifest: Asset;
    type RawItem;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        _world: &World,
    ) -> Result<Self, Self::Err>;
}
