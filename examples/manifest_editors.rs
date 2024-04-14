//! Modifying manifest files by hand can be error-prone, especially as the complexity of the manifest grows.
//! Instead, you should consider building tooling to help you manage your manifests and allow game designers to work with them more easily.
//!
//! The core pattern, demonstrated here, is quite simple!
//!
//! 1. Load the manifest file into memory, converting it from a raw manifest to its final manifest form.
//! 2. Modify the manifest in memory, using GUI or command-line tools. Reflection is very useful here!
//! 3. Save the manifest back to disk, converting it from its final manifest form back to a raw manifest.
//!
//! Validation is also important, and can and should be performed at each step of the process.

fn main() {}
