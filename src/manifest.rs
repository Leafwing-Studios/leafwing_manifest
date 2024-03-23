use std::error::Error;

use bevy::{asset::Asset, ecs::world::World};

pub trait Manifest: Sized + 'static {
    type Item;
    type Err: Error;
    type RawManifest: Asset;
    type RawItem;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        _world: &mut World,
    ) -> Result<Self, Self::Err>;
}
