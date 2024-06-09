//! This PR demonstrates how to spawn entities based on manifest entries by constructing a bundle based on the data in the manifest.
//!
//! While there are several possible ways you could achieve this goal, we've found that simply creating a custom bundle
//! and defining a constructor that takes the manifest data is a very effective way to ensure that all of your entities
//! have the right components, no matter where they're spawned.
//!
//! Generally speaking, you'll want to create a custom bundle type for each manifest that you have.
//! Store a handle to *all* of the assets that you need in the bundle:
//! this will allow you to avoid passing in references to the asset storage at every call site.
//!
//! If you need to spawn a scene hierarchy (such as for levels or 3D models), storing a handle to that scene can work well,
//! or a scene bundle can be added to your custom bundle type.

use bevy::{prelude::*, sprite::Mesh2dHandle, utils::HashMap};
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
    plugin::{ManifestPlugin, RegisterManifest},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RawTile {
    name: String,
    /// An RGB color in float form.
    color: [f32; 3],
    // Serializing enums works just fine,
    // and there are often *some* properties that should be an exhaustive list of options.
    tile_type: TileType,
}

#[derive(Debug)]
pub struct Tile {
    name: String,
    // We convert the supplied u32 color into a `ColorMaterial` during manifest processing.
    color_material: Handle<ColorMaterial>,
    // The same square mesh is used for all tiles,
    // and can be procedurally generated during .
    mesh: Mesh2dHandle,
    tile_type: TileType,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
enum TileType {
    City,
    Water,
    Wilderness,
}

// Creating a custom bundle allows us to ensure that all of our tile objects have the right components,
// no matter where they're spawned.
#[derive(Bundle)]
pub struct TileBundle {
    // Storing the `Id<Tile>` directly on the bundle allows us to easily look up particularly heavy data later.
    // It also serves as a nice way to filter for tiles in queries.
    id: Id<Tile>,
    tile_type: TileType,
    material: Handle<ColorMaterial>,
    mesh: Mesh2dHandle,
    // Add all of the components needed to render the tile.
    spatial_bundle: SpatialBundle,
}

impl TileBundle {
    // When defining constructors, you'll typically find that you need to pass in both
    // the manifest data (describing the fundamental properties of the entity)
    // and information about the exact location and dynamic properties of the entity required.
    // Other fields (such as Visibility here) will *always* be the same,
    // so we don't need to duplicate the data in the manifest.
    fn new(transform: Transform, tile: &Tile) -> Self {
        Self {
            id: Id::from_name(&tile.name),
            tile_type: tile.tile_type,
            // We can use weak clones here and save a tiny bit of work,
            // since the manifest will always store a canonical strong handle to the assets.
            material: tile.color_material.clone_weak(),
            // While the value of the mesh is the same for all tiles, passing around `&Assets<Mesh>` everywhere
            // is miserable. Instead, we sacrifice a little bit of memory to redundantly store the mesh handle in the manifest:
            // like always, the mesh itself is only stored once in the asset storage.
            mesh: tile.mesh.clone(),
            spatial_bundle: SpatialBundle::from_transform(transform),
        }
    }
}

#[derive(Asset, Serialize, Deserialize, TypePath, Debug, PartialEq)]
pub struct RawTileManifest {
    tiles: Vec<RawTile>,
}

#[derive(Resource, Default)]
pub struct TileManifest {
    tiles: HashMap<Id<Tile>, Tile>,
}

impl Manifest for TileManifest {
    type Item = Tile;
    type RawItem = String;
    type RawManifest = RawTileManifest;
    type ConversionError = std::convert::Infallible;

    const FORMAT: ManifestFormat = ManifestFormat::Ron;

    fn get(&self, id: Id<Tile>) -> Option<&Self::Item> {
        self.tiles.get(&id)
    }

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        world: &mut World,
    ) -> Result<Self, Self::ConversionError> {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(Mesh::from(Rectangle::new(1.0, 1.0)));
        // This is a thin wrapper around a `Handle<Mesh>`, used in 2D rendering.
        let mesh_2d = Mesh2dHandle::from(mesh.clone());

        let mut color_materials = world.resource_mut::<Assets<ColorMaterial>>();

        let mut manifest = TileManifest::default();

        for raw_tile in raw_manifest.tiles {
            // This is a very simple example of procedurally generated assets,
            // driven by hand-tuned parameters in the manifest.
            // In a real game, you might use a more complex system to generate the assets,
            // but the general pattern is very effective for creating cohesive but varied content.
            let color_material = color_materials.add(Color::srgb_from_array(raw_tile.color));

            manifest.tiles.insert(
                Id::from_name(&raw_tile.name),
                Tile {
                    name: raw_tile.name,
                    color_material,
                    // We need to store strong handles here: otherwise the procedural mesh will be dropped immediately
                    // when the original declaration goes out of scope.
                    mesh: mesh_2d.clone(),
                    tile_type: raw_tile.tile_type,
                },
            );
        }

        Ok(manifest)
    }
}

pub fn spawn_tiles(mut commands: Commands, tile_manifest: Res<TileManifest>) {
    // 2D camera scales are measured in pixels per unit.
    const SCALE: f32 = 128.;
    // Space the tiles out a bit.
    const SPACING: f32 = 1.5;

    info!("Spawning tiles...");

    // Remember to add the camera bundle to the world, or you won't see anything!
    commands.spawn(Camera2dBundle::default());

    for (i, tile) in tile_manifest.tiles.values().enumerate() {
        info!("Spawning tile: {:?}", tile);

        // Space out the spawned tiles for demonstration purposes.
        let translation = Vec3::X * i as f32 * SCALE * SPACING;
        let transform = Transform::from_translation(translation).with_scale(Vec3::splat(SCALE));

        commands.spawn(TileBundle::new(transform, tile));
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<SimpleAssetState>()
        .add_plugins(ManifestPlugin::<SimpleAssetState>::default())
        .register_manifest::<TileManifest>("tiles.ron")
        .add_systems(OnEnter(SimpleAssetState::Ready), spawn_tiles)
        .run();
}

/// This module is used to generate the tile manifest.
///
/// While manifests *can* be hand-authored, it's often more convenient to generate them using tooling of some kind.
/// Serde's [`Serialize`] and [`Deserialize`] traits are a good fit for this purpose.
/// `ron` is a straightforward human-readable format that plays well with Rust's type system, and is a good point to start.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_raw_item_manifest() {
        let mut raw_tile_manifest = RawTileManifest {
            tiles: Vec::default(),
        };

        raw_tile_manifest.tiles.push(RawTile {
            name: "Gotham".to_string(),
            color: [0.0, 1.0, 0.0],
            tile_type: TileType::City,
        });

        raw_tile_manifest.tiles.push(RawTile {
            name: "River".to_string(),
            color: [0.0, 0.0, 1.0],
            tile_type: TileType::Water,
        });

        raw_tile_manifest.tiles.push(RawTile {
            name: "Dark Forest".to_string(),
            color: [1.0, 0.0, 0.0],
            tile_type: TileType::Wilderness,
        });

        let serialized =
            ron::ser::to_string_pretty(&raw_tile_manifest, Default::default()).unwrap();
        println!("{}", serialized);

        // Save the results, to ensure that our example has a valid manifest to read.
        std::fs::write("assets/tiles.ron", &serialized).unwrap();

        let deserialized: RawTileManifest = ron::de::from_str(&serialized).unwrap();

        assert_eq!(raw_tile_manifest, deserialized);
    }
}
