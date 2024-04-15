# leafwing_manifest

`leafwing_manifest` is a straightforward, opinionated tool to transform "assets on disk" into flexible, robust objects inside of your Bevy game.

## Usage

`leafwing_manifest` has four key concepts:

1. **Id:** A unique identifier for objects of a given class (e.g. monsters, tile types or levels). Commonly stored as a components on game entities.
2. **Item:** An in-memory representation of all of the shared data (e.g. name, asset handles, statistics) of game objects of a given kind.
3. **Manifest:** a Bevy `Resource` which contains a mapping from identitifiers to items.
4. **Raw manifest:** a serialization-friendly representation of the data stored in a manifest.

Data is deserialized from disk into a raw manifest, which is processed into a manifest which contains a list of all available game objects of a given class.
That manifest is then used to spawn and look up the properties of specific kinds of game objects in your game code.

To get started:

1. Add an asset loading state that implements `AssetState` to your app that handles the lifecycle of loading assets.
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

## Motivation

When you first started building your game, you may have modelled every content option as an enum. Character classes, buildings, towns, platformer levels, loot...
This is great for prototyping: enums are simple to set up, and hard to screw up because of exhaustive pattern matching.

However, enums require exhaustive enumeration. Adding another item type requires adding an arm to *everywhere* that this is being used, and you can't just add more items off in their own module without updating every single call site.

So instead, we could use traits! Traits are extensible: great for this sort of "open set with shared properties" abstraction.
While traits can be a nuisance (you almost certainly need to ensure your trait is object-safe!), they can be a nice fit for a lot of games or forms of content.

But then you realize either:

1. You want your game to be easily moddable, by people who don't have access to your source code and don't know Rust.
2. You have artists and game designers on your team who want to be able to just quickly author new content without having to change the code at all.

There's a standard solution to this: **data-driven** content.
Rather than defining your variants in code, define their properties using data that you can store on a hard drive,
and then use the variations in the supplied fields to create a rich variation in gameplay by parsing these properties inside your game.

## Basics of implementing data-driven content

In a data-driven design architecture, data must be transformed from the **serialized format**, which is stored on the disk, into the **in-game representation**, which is used by game systems and logic to actually interact in the game world.

We can start with a very simple architecture:

1. Store a string corresponding to each type of game object that matches the file name of the serialized data.
2. When we want to spawn an object of that type, pass in that string.
3. During the spawning process, deserialize that file directly into a game object.
4. Spawn the generated game object.

## More sophisticated data-driven content

Now, there are several areas for improvement.
Let's go over them now, and then tackle them one at a time:

1. Strings are heavy identifiers: as long as we can convert to and from a nice human-readable string, we don't care about actually storing that identifier everywhere.
2. Constantly deserializing from disk is very slow.
3. Similarly, some final game objects are very heavy: we want to look up the data when we need it, rather than storing thousands of copies of it for every object.
4. We can't always translate directly from the serialized to in-game representation. This is particularly common and important when we need to reference *other* serialized content inside of our game object. It also comes up when you want to simplify the serialized representation to make it more space efficient or easier to author.

There are a few additional problems, which Leafwing has chosen to leave as "future work" for now:

1. Text-based file formats are easy to mess up: tools for validating these would be very useful.
2. Text-based file formats are easy to mess up: it would be great to have designer-friendly tools for authoring them.

### Efficient identifiers

Strings are a very natural fit for identifiers in the context of game development:
every asset needs its own file name anyways, and we need to be able to talk about our different bits of content.

However, strings are relatively slow and cumbersome and initial experiments quickly ran into performance problems.
But what makes a good identifier?
Identifiers should be:

1. **Compact:** taking up space hurts both memory usage and the speed of all operations.
2. **Stable:** adding another game object shouldn't break all of your existing IDs.
3. **Low-collision:** game objects should have a low risk of colliding with each other.
4. **Const-constructable:** defining constants representing a game object reduces the risk of typos and eases refactoring.
5. **Type-safe:** it should be impossible to accidently use the id for a monster where the id for a piece of loot is called for.

To achieve the first four properties, we hash the names of our game objects, using the stored hashes as our opaque identifiers.
Finally, we achieve some degree of type-safety by adding a marker generic to each `Id`, which corresponds to the type of manifest that it indexes.

### Object storage deduplication

We can use the generated manifest to meet two of our other goals: avoiding constant deserialization and deduplicating heavy data.

The manifest (or a central asset server) holds a single, canonical copy of each game object.

Depending on your needs, you can either look up the value as it is needed (using the unique key), or copy it directly onto your game object, generally as components.
The latter is generally going to result in faster iteration (and random access via `query.get`), but will lead to increased memory costs.

### Intermediate deserialization formats

Jumping directly from the serialized form to the in-game form of an asset is often not particularly feasible or fun. For example:

1. Data-driven game objects commonly want to reference *other* data-driven game object.
This comes up when dealing with things like "loot drop tables", "enemy spawn rates for each level", or "allowable terrain for buildings".
2. You may need to reference other assets, such as meshes or sound effects, that require a `Handle` from Bevy's `AssetServer`.
3. Parsing and storing enum-like values from string-based formats like JSON can be fraught with footguns.

Rather than encouraging users to build progressively more complex (and fragile) serializers and deserializers, `leafwing_manifest` takes a pragmatic approach:

1. For each manifest, users may create a permissive, intermediate **raw manifest** data structure that maps directly to the constraints of the underlying file format (e.g. CSV or JSON). If you need a manual `Deserialize` impl it's too complicated.
2. Load all of the raw manifests first.
3. Once all raw manifests are loaded, attempt to convert them into their final **processed manifest** forms, cross-referencing other manifests as needed.

This conversion process is flexible and direct, and involves ordinary synchronous Rust code. Here's the heart of the `Manifest` trait:

```rust
use bevy::prelude::*;
use std::error::Error;

trait Manifest: Sized + Resource {
    /// The raw data type that is loaded from disk.
    type RawManifest: Asset + for<'de> Deserialize<'de>;

    /// The raw data type that is stored in the manifest.
    type RawItem;

    /// The type of the game object stored in the manifest.
    type Item;

    /// The error type that can occur when converting raw manifests into a manifest.
    type ConversionError: Error;

    /// The format of the raw manifest on disk.
    const FORMAT: ManifestFormat;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        _world: &mut World,
    ) -> Result<Self, Self::Err>;
}
```

If you don't require this intermediate step, you can simply leave it out! Set `Err = Infallible`, `RawManifest=Self` and `RawItem=Item` to avoid added work or complexity.

If you've ever written a `TryFrom` impl in Rust before, this should be very familiar to you.
Attempt to convert the items in the collection, one at a time, returning errors via the `?` operator as needed.
The only wrinkle is that we *also* have access to the entire Bevy `World`, allowing us to access asset data or other manifests as needed.
