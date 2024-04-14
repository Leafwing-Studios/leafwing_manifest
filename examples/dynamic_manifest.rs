//! While manifests are typically defined once and then used throughout the lifetime of the application,
//! there are cases where you may want to dynamically generate or modify a manifest at runtime.
//!
//! For example, you may be working with user-generated content, have a modding system that allows players to add new content to the game,
//! or simply have more content than you can fit into memory at once.
//!
//! By mutating the contents of our manifest at runtime, we can add (and remove!) game objects to the game world as needed.

fn main() {}

/*
    fn get_mut(&mut self, id: &Id<Item>) -> Option<&mut Self::Item> {
        self.items.get_mut(id)
    }

    fn insert(
        &mut self,
        item: Self::Item,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>> {
        // Names can be used as unique identifiers for items;
        // the from_name method quickly hashes the string into a unique ID.
        let id = Id::from_name(item.name.clone());

        // Because we're relying on the name as a unique identifier,
        // we need to check for duplicates.
        if self.items.contains_key(&id) {
            Err(ManifestModificationError::DuplicateName(item.name.clone()))
        } else {
            self.items.insert(id, item);
            Ok(id)
        }
    }

    fn remove(
        &mut self,
        id: &Id<Self::Item>,
    ) -> Result<Id<Self::Item>, ManifestModificationError<Self>> {
        self.items.remove(id);
        Ok(*id)
    }
*/
