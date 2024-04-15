# leafwing_manifest

`leafwing_manifest` is a straightforward, opinionated tool to transform "assets on disk" into flexible, robust objects inside of your Bevy game.

For more background reading on why this crate exists, and the design decisions made, check out [`MOTIVATION.md`].

## Usage

`leafwing_manifest` has four key concepts:

1. **Id:** A unique identifier for objects of a given class (e.g. monsters, tile types or levels). Commonly stored as a components on game entities.
2. **Item:** An in-memory representation of all of the shared data (e.g. name, asset handles, statistics) of game objects of a given kind.
3. **Manifest:** a Bevy `Resource` which contains a mapping from identitifiers to items.
4. **Raw manifest:** a serialization-friendly representation of the data stored in a manifest.

Data is deserialized from disk into a raw manifest, which is processed into a manifest which contains a list of all available game objects of a given class.
That manifest is then used to spawn and look up the properties of specific kinds of game objects in your game code.

To get started:

1. Add an asset loading state that implements `AssetState` (like the `SimpleAssetState` that we ship) to your app that handles the lifecycle of loading assets.
2. Add `ManifestPlugin<S: AssetState>` to your `App`.
3. Create a struct (e.g. `Monster`) that stores the final data that you want to share between all objects of the same kind.
4. Put your game design hat on and define each monster's life, level, name and so on in a serialized format like RON.
5. Register the manifest in your app with `app.register_manifest::<Monster>`, supplying the path to the data to load.
6. In your game logic, spawn monsters or look up their statistics by calling `Manifest<Monster>.get(id: Id<Monster>)`.

See the `simple.rs` example to jump right in!

If your assets require processing (for validation, or if they contain references to other assets),
you will need a raw manifest type, and corresponding raw item type.
Take a look at the `raw_manifest.rs` example next!

Note that we *don't* compress our manifests into a binary format in our examples.
While you *can* do so, we don't encourage you to (except as an optimization in shipped games).
The added pain during version control and debugging is typically not worth the improvements to file size or loading speed during development.
