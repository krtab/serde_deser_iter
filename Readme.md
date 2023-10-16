This crate offers two different ways to deserialize sequences without
allocating.

# Example

Given the following JSON:

```json
[
    {"id": 0, "name": "bob", "subscribed_to": ["rust", "knitting", "cooking"]},
    {"id": 1, "name": "toby 🐶", "subscribed_to": ["sticks", "tennis-balls"]},
    {"id": 2, "name": "alice", "subscribed_to": ["rust", "hiking", "paris"]},
    {"id": 3, "name": "mark", "subscribed_to": ["rust", "rugby", "doctor-who"]},
    {"id": 4, "name": "vera", "subscribed_to": ["rust", "mma", "philosophy"]}
]
```
we can process it without allocating a 5-sized vector of items as follow:

```rust
use serde_iter::top_level::DeserializerExt;
# use std::{fs::File, io::BufReader, path::PathBuf, collections::HashSet};
#
/// The type each item in the sequence will be deserialized to.
#[derive(serde::Deserialize)]
struct DataEntry {
    // Not all fields are needed, but we could add "name"
    // and "id".
    subscribed_to: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    #
    # let example_json_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "examples", "data.json"]
    #   .iter()
    #   .collect();
    let buffered_file: BufReader<File> = BufReader::new(File::open(example_json_path)?);
    let mut json_deserializer = serde_json::Deserializer::from_reader(buffered_file);
    let mut all_channels = HashSet::new();

    json_deserializer.for_each(|entry: DataEntry| all_channels.extend(entry.subscribed_to))?;
    println!("All existing channels:");
    for channel in all_channels {
        println!("  - {channel}")
    }
    Ok(())
}
```

# Top-level vs deep

## Top-level

The top_level module offers the most user friendly and powerful way to
deserialize sequences. However, it is restricted to sequences defined at
the top-level. For example it can work on each `{"name": ...}` from the following JSON

```json
[
    {"name": "object1"},
    {"name": "object2"},
    {"name": "object3"}
]
```

but not if they are deeper in the structure:

```json
{
    "result": [
        {"name": "object1"},
        {"name": "object2"},
        {"name": "object3"}
    ]
}
```

## Deep

The deep module allows working on sequences located at any depth
(and even nested one, though cumbersomely). However it does not allow to
run closures on the iterated items, only functions, and its interface is
less intuitive than top_level.