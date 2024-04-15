# leafwing_manifest

`leafwing_manifest` is a straightforward, opinionated tool to transform "assets on disk" into flexible, robust objects inside of your Bevy game.

## Usage

TODO.

## Motivation

So, throughout your game, you're modeling content options as an enum. Classes, buildings, enum types, levels, items...

This is really great for prototyping: enums are simple to set up, and hard to screw up because of exhaustive pattern matching.

However, they also require exhaustive enumeration. Adding another item type requires adding an arm to *everywhere* that this is being used, and you can't just add more items off in their own file without extending this.

So instead, we could use traits! Traits are extensible: great for this sort of "open set with shared properties" abstraction.
While traits can be a nuisance (you almost certainly need to ensure your trait is object-safe!), they can be a nice fit for a lot of games or content areas.

But then you realize either:

1. You want your game to be easily moddable, by people who don't have access to your source code and don't know Rust.
2. You have artists and game designers on your team who want to be able to just quickly author new content without having to change the code at all.

There's a standard solution to this: **data-driven** content.
Rather than defining your variants in code, define their properties using data that you can store on a hard drive,
and then use the variations in the supplied fields to create a rich variation in gameplay by parsing these properties inside your game.

## Basics of implementing data-driven content

In a data-driven design architecture, data must be transformed from the **serialized format**, stored on the disk, into the **in-game representation**, used by game systems and logic to actually interact in the game world.

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
3. Some final game objects are very heavy: we want to look up the data when we need it, rather than storing thousands of copies of it for every object.
4. We can't always translate directly from the serialized to in-game representation. This is particularly common and important when we need to reference *other* serialized content inside of our game object. It also comes up when you want to simplify the serialized representation to make it more space efficient or easier to author.

In "problems left for future work":

1. Text-based file formats are easy to mess up: tools for validating these would be very useful.
2. Text-based file formats are easy to mess up: it would be great to have designer-friendly tools for authoring them.

Note that we *don't* compress our manifests into a binary format in our examples, and while you *can* do so, we don't encourage you to (except perhaps in shipped games).
The added pain during version control and debugging is not worth the space savings (or benchmark-requiring loading speed improvements).

### Efficient identifiers

Simplifying our identifiers is pretty easy.
All we need to do is to store a bidirectional map between the identifiers and our handy strings!
In this crate, we call this map a **manifest**: it's an exhaustive listing of goods!

But what makes a good identifier?
Identifiers should be:

1. Compact: they shouldn't take up much space
2. Stable: adding another game object shouldn't break all of your existing IDs
3. Low-collision: game objects should have a low risk of colliding with

To achieve these properties, we hash the strings, using the stored hashes as our opaque identifiers!

### Object storage deduplication

We can use the generated manifest to meet two of our other goals: avoiding constant deserialization and deduplicating heavy data.

The manifest (or a central asset server) holds a single, canonical copy of each game object.

Depending on your needs, you can either look up the value as it is needed (using the unique key), or copy it directly into your game object.

### Intermediate deserialization formats

Jumping directly from the serialized form to the in-game form of an asset is often not particularly feasible or fun. For example:

1. Data-driven game objects commonly want to reference *other* data-driven game objects, and the patterns here don't always obey nice clean tree structures.
This comes up when dealing with things like "loot drop tables", "enemy spawn rates for each level", or "allowable terrain for buildings".
2. You may need to reference other assets, such as meshes or sound effects, that you aren't sure will exist.
3. Parsing and storing enum-like values from string-based formats like JSON can be fraught with footguns.

Rather than encouraging users to build progressively more complex (and fragile) serializers and deserializers, `leafwing_manifest` takes a pragmatic approach:

1. For each manifest, users may create a permissive, intermediate **raw manifest** data structure that maps directly to the constraints of the underlying file format (e.g. JSON).
2. Load all of the raw manifests first.
3. Once all raw manifests are loaded, attempt to convert them into their **processed manifest** forms, cross-referencing other manifests as needed.

This conversion process is flexible and direct, and involves ordinary synchronous Rust code. Here's the heart of the `Manifest` trait:

```rust
use bevy::prelude::*;
use std::error::Error;

trait Manifest: Sized {
    type Item;
    type Err: Error;
    type RawManifest;
    type RawItem;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        _world: &mut World,
    ) -> Result<Self, Self::Err>;
}
```

If you don't require this intermediate step, you can simply leave it out! Set `Err = ()`, `RawManifest=Self` and `RawItem=Item` to avoid added work or complexity.

If you've ever written a `TryFrom` impl in Rust before, this should be very familiar to you.
Attempt to convert the items in the collection, one at a time, returning errors using the `?` operator as needed.
The only wrinkle is that we *also* have access to the entire Bevy `World`, allowing us to access (and process) other manifests as needed.
