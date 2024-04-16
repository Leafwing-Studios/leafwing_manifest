use bevy::{prelude::*, ui::ContentSize, utils::HashMap};
use leafwing_manifest::{
    asset_state::SimpleAssetState,
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
    plugin::{AppExt, ManifestPlugin},
};
use serde::{Deserialize, Serialize};

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<SimpleAssetState>()
        .add_plugins(ManifestPlugin::<SimpleAssetState>::default())
        .register_manifest::<DialogBoxManifest>("dialog_boxes.ron")
        .add_systems(Startup, (spawn_dialog_boxes,))
        .run();
}
