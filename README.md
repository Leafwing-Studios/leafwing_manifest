# leafwing_manifest

`leafwing_manifest` is a straightforward, opinionated tool to transform "assets on disk" into flexible, robust objects inside of your Bevy game.
There are four key concepts:

1. **Id:** A unique identifier for objects of a given class (e.g. monsters, tile types or levels). Commonly stored as a components on game entities.
2. **Item:** An in-memory representation of all of the shared data (e.g. name, asset handles, statistics) of game objects of a given kind.
3. **Manifest:** a Bevy `Resource` which contains a mapping from identitifiers to items.
4. **Raw manifest:** a serialization-friendly representation of the data stored in a manifest.

Data is deserialized from disk into a raw manifest, which is processed into a manifest which contains a list of all available game objects of a given class.
That manifest is then used to spawn and look up the properties of specific kinds of game objects in your game code.

## Why manifests rock

An in-memory resource where you can look up the statistics for various game objects is incredibly useful:

1. It's super easy to spawn new game objects dynamically inside of gameplay code, including spawning multiple copies of an object well after the initial asset load. Simply write a helper method once, and then spawn any object you want in a single command.
2. Spawning multiple clones of the same gameplay object is safer. Since the manifest is generally read-only, you know you always have a "clean" version to clone from.
3. Manifests offer a clear list of all objects of a given kind: great for both dev tools and in-game encyclopedias.
4. Using manifests abstracts away messy asset loading (and unloading) code into a single consistent pattern that can grow with your project.
5. Heavy data can be deduplicated by simply looking it up in the manifest when needed.

For more background reading on why this crate exists, and the design decisions made, check out [`MOTIVATION.md`](https://github.com/Leafwing-Studios/leafwing_manifest/blob/main/MOTIVATION.md).

## Usage

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
