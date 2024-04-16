use bevy::{prelude::*, utils::HashMap};
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
    plugin::{AppExt, ManifestPlugin},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RawTile {
    name: String,
    /// An RGB color in float form.
    color: [f32; 3],
}

pub struct Tile {
    name: String,
    // We convert the supplied u32 color into a `ColorMaterial` during manifest processing.
    color_material: Handle<ColorMaterial>,
}

// Creating a custom bundle allows us to ensure that all of our tile objects have the right components,
// no matter where they're spawned.
#[derive(Bundle)]
pub struct TileBundle {
    id: Id<Tile>,
    material: Handle<ColorMaterial>,
    mesh: Handle<Mesh>,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    transform: Transform,
    global_transform: GlobalTransform,
}

impl TileBundle {
    // By adding custom constructors that take a `Tile` object, we can quickly look up the right data for each tile,
    // and easily update the call sites as we add more fields to the `Tile` struct.
    fn new(tile: &Tile, transform: Transform) -> Self {
        Self {
            id: Id::from_name(&tile.name),
            material: tile.color_material.clone(),
            mesh: Default::default(),
            visibility: Default::default(),
            inherited_visibility: Default::default(),
            transform,
            global_transform: Default::default(),
        }
    }
}

#[derive(Asset, Serialize, Deserialize, TypePath)]
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
        let mut color_materials = world.resource_mut::<Assets<ColorMaterial>>();

        let mut manifest = TileManifest::default();

        for raw_tile in raw_manifest.tiles {
            // This is a very simple example of procedurally generated assets,
            // driven by hand-tuned parameters in the manifest.
            // In a real game, you might use a more complex system to generate the assets,
            // but the general pattern is very effective for creating cohesive but varied content.
            let color_material = color_materials.add(Color::rgb_from_array(raw_tile.color));

            manifest.tiles.insert(
                Id::from_name(&raw_tile.name),
                Tile {
                    name: raw_tile.name,
                    color_material,
                },
            );
        }

        Ok(manifest)
    }
}

pub fn spawn_tiles(mut commands: Commands, tile_manifest: Res<TileManifest>) {
    for (i, tile) in tile_manifest.tiles.values().enumerate() {
        // Space out the spawned tiles arbitrarily.
        let translation = Vec3::X * i as f32;
        let transform = Transform::from_translation(translation);

        commands.spawn(TileBundle::new(tile, transform));
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<SimpleAssetState>()
        .add_plugins(ManifestPlugin::<SimpleAssetState>::default())
        .register_manifest::<TileManifest>("tiles.ron")
        .add_systems(Startup, spawn_tiles)
        .run();
}
