use std::error::Error;

use bevy::ecs::world::World;

pub trait Manifest: Sized {
    type Item;
    type Err: Error;
    type RawManifest;
    type RawItem;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        _world: &mut World,
    ) -> Result<Self, Self::Err>;
}
