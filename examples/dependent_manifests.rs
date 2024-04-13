//! Data is rarely nicely isolated. Often, you may need to describe how data of one sort (e.g. monsters) relates to data of another sort (e.g. items or levels).
//!
//! In this example, a level may contain monsters, items and other game objects.
//! A monster may in turn reference an item, as part of its loot table.
//!
//! By carefully controlling the order in which manifests are processed using system ordering, you can ensure that all the required data is available when you need it.

fn main() {}
