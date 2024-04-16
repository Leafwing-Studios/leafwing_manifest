use bevy::{prelude::*, utils::HashMap};
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
    plugin::{AppExt, ManifestPlugin},
};
use serde::{Deserialize, Serialize};

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<SimpleAssetState>()
        .add_plugins(ManifestPlugin::<SimpleAssetState>::default())
        .register_manifest::<AnimalManifest>("animals.ron")
        .add_systems(Startup, (spawn_animals,))
        .run();
}
