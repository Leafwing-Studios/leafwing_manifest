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

//! When working with manifests, you may find that the most convenient format for serialization is not the most convenient format for working with in code.
//!
//! For example, you may need to cross-reference other items in the manifest, perform some kind of validation on the data, or store handles to assets.
//! While we *could* perform these operations every time we need to access the data, it's more efficient to perform them once and store the results.
//!
//! This is where raw manifests come in.
//! Serialized data is converted to raw manifests via asset loading,
//! and then processed into the final manifests which can be accessed as resources to look up the properties of your items.
//!
//! This example showcases a relatively simple raw manifest pattern, where the raw manifest is used to initialize handles to sprites;
//! see the other examples for more complex use cases!
//! This example builds on the `simple.rs` example, using much of the same code and patterns.

use bevy::{prelude::*, utils::HashMap};
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
    plugin::{AppExt, ManifestPlugin},
};
use serde::{Deserialize, Serialize};

/// This module demonstrates the item-as-bundle pattern.
///
/// This is the simplest approach to spawning entities from a manifest,
/// but is not very flexible as your needs change.
mod item_as_bundle {
    use core::str;

    use bevy::ui::ContentSize;

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct RawDialogBox {
        // If you were using a localization solution like fluent,
        // you might store a key here instead of the actual text
        // and use that as the name as well.
        name: String,
        text: String,
    }

    #[derive(Bundle)]
    pub struct DialogBox {
        text_bundle: TextBundle,
    }

    // TextBundle doesn't implement Clone :(
    // Tracked in <https://github.com/bevyengine/bevy/issues/12985>
    impl Clone for DialogBox {
        fn clone(&self) -> Self {
            Self {
                text_bundle: TextBundle {
                    node: self.text_bundle.node.clone(),
                    text_layout_info: self.text_bundle.text_layout_info.clone(),
                    text_flags: self.text_bundle.text_flags.clone(),
                    // This field isn't `Clone`, but that's fine because it will be recomputed
                    calculated_size: ContentSize::default(),
                    focus_policy: self.text_bundle.focus_policy.clone(),
                    text: self.text_bundle.text.clone(),
                    style: self.text_bundle.style.clone(),
                    transform: self.text_bundle.transform.clone(),
                    global_transform: self.text_bundle.global_transform.clone(),
                    visibility: self.text_bundle.visibility.clone(),
                    inherited_visibility: self.text_bundle.inherited_visibility.clone(),
                    view_visibility: self.text_bundle.view_visibility.clone(),
                    z_index: self.text_bundle.z_index.clone(),
                    background_color: self.text_bundle.background_color.clone(),
                },
            }
        }
    }

    impl DialogBox {
        fn from_raw(raw: &RawDialogBox) -> Self {
            Self {
                text_bundle: TextBundle::from_section(raw.text.clone(), TextStyle::default()),
            }
        }
    }

    #[derive(Asset, Serialize, Deserialize, TypePath)]
    pub struct RawDialogBoxManifest {
        dialog_boxes: Vec<RawDialogBox>,
    }

    #[derive(Resource)]
    pub struct DialogBoxManifest {
        dialog_boxes: HashMap<Id<DialogBox>, DialogBox>,
    }

    impl Manifest for DialogBoxManifest {
        type Item = DialogBox;
        type RawItem = RawDialogBox;
        type RawManifest = RawDialogBoxManifest;
        type ConversionError = std::convert::Infallible;

        const FORMAT: ManifestFormat = ManifestFormat::Ron;

        fn get(&self, id: Id<DialogBox>) -> Option<&Self::Item> {
            self.dialog_boxes.get(&id)
        }

        fn from_raw_manifest(
            raw_manifest: Self::RawManifest,
            _world: &mut World,
        ) -> Result<Self, Self::ConversionError> {
            let mut dialog_boxes = HashMap::default();

            for raw_dialog_box in raw_manifest.dialog_boxes.iter() {
                dialog_boxes.insert(
                    Id::from_name(&raw_dialog_box.name),
                    DialogBox::from_raw(raw_dialog_box),
                );
            }

            Ok(Self { dialog_boxes })
        }
    }

    pub fn spawn_dialog_boxes(mut commands: Commands, dialog_box_manifest: Res<DialogBoxManifest>) {
        for dialog_box in dialog_box_manifest.dialog_boxes.values() {
            commands.spawn(dialog_box.clone());
        }
    }
}

/// This module demonstrates the item-as-partial-bundle pattern.
///
/// This pattern is useful when you have a lot of duplicated data between items in the manifest.
mod items_as_partial_bundle {
    use super::*;

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
}

/// This module demonstrates the item-as-scene pattern.
///
/// When spawning entities in this way, we insert the scene into the world as a whole, rather than adding individual components on a single entity.
/// Note that we could use a `SceneBundle` instead of a `Scene` if we preferred.
mod item_as_scene {
    use super::*;
    use bevy::scene::Scene;

    #[derive(Serialize, Deserialize)]
    pub struct RawAnimal {
        name: String,
        movement_speed: f32,
    }

    #[allow(dead_code)]
    pub struct Animal {
        name: String,
        movement_speed: f32,
        // This field is not present in the raw data,
        // instead, we infer the path to the corresponding asset from the name,
        // generating it from a gltF file in the `assets/models` directory.
        scene: Handle<Scene>,
    }

    #[derive(Asset, Serialize, Deserialize, TypePath)]
    pub struct RawAnimalManifest {
        animals: Vec<RawAnimal>,
    }

    #[derive(Resource, Default)]
    pub struct AnimalManifest {
        animals: HashMap<Id<Animal>, Animal>,
    }

    impl Manifest for AnimalManifest {
        type Item = Animal;
        type RawItem = RawAnimal;
        type RawManifest = RawAnimalManifest;
        // Assigning handles is an infallible operation,
        // even though those handles can later be invalidated.
        type ConversionError = std::convert::Infallible;

        const FORMAT: ManifestFormat = ManifestFormat::Ron;

        fn get(&self, id: Id<Animal>) -> Option<&Self::Item> {
            self.animals.get(&id)
        }

        fn from_raw_manifest(
            raw_manifest: Self::RawManifest,
            world: &mut World,
        ) -> Result<Self, Self::ConversionError> {
            let asset_server = world.resource::<AssetServer>();

            let mut manifest = AnimalManifest::default();

            for raw_animal in raw_manifest.animals {
                let scene = asset_server.load(format!("models/{}.gltf", raw_animal.name));

                manifest.animals.insert(
                    Id::from_name(&raw_animal.name),
                    Animal {
                        name: raw_animal.name,
                        movement_speed: raw_animal.movement_speed,
                        scene,
                    },
                );
            }

            Ok(manifest)
        }
    }

    pub fn spawn_animals(
        mut scene_spawner: ResMut<SceneSpawner>,
        animal_manifest: Res<AnimalManifest>,
    ) {
        for animal in animal_manifest.animals.values() {
            scene_spawner.spawn(animal.scene.clone());
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<SimpleAssetState>()
        .add_plugins(ManifestPlugin::<SimpleAssetState>::default())
        .register_manifest::<item_as_bundle::DialogBoxManifest>("dialog_boxes.ron")
        .register_manifest::<items_as_partial_bundle::TileManifest>("tiles.ron")
        .register_manifest::<item_as_scene::AnimalManifest>("animals.ron")
        .add_systems(
            Startup,
            (
                item_as_bundle::spawn_dialog_boxes,
                items_as_partial_bundle::spawn_tiles,
                item_as_scene::spawn_animals,
            ),
        )
        .run();
}
