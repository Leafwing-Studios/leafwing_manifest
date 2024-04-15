//! When generating content through a combination of code and assets,
//! it is often useful to be able to quickly spawn new instances of objects by name from inside the code.
//!
//! This workflow tends to make the most sense during prototyping, where the exact details of the content are still in flux,
//! or when the content is generated procedurally and the exact details are not known ahead of time.
//! By contrast, traditional "levels" generally make most sense as pure assets,
//! although even then it can be useful to be able to spawn objects by name for scripting purposes:
//! creating effects, dynamically spawning enemies, etc.
//!
//! This example shows how to use manifests to create a simple system for spawning objects by name,
//! although the same principles can be used to manipulate manifests if using the [`MutableManifest`] trait as well.

fn main() {}
