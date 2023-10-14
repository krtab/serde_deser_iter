use serde_iter::DeserializerExt;
use std::{fs::File, io::BufReader, path::PathBuf, collections::HashSet};

#[derive(serde::Deserialize)]
struct DataEntry {
    subscribed_to: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let example_json_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "examples", "data.json"]
        .iter()
        .collect();
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
