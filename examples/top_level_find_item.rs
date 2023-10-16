use serde_deser_iter::top_level::DeserializerExt;
use std::{fs::File, io::BufReader, path::PathBuf};

#[derive(serde::Deserialize)]
struct DataEntry {
    id: u32,
    name: String,
    subscribed_to: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let example_json_path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "examples",
        "top_level_data.json",
    ]
    .iter()
    .collect();
    let buffered_file: BufReader<File> = BufReader::new(File::open(example_json_path)?);
    let mut json_deserializer = serde_json::Deserializer::from_reader(buffered_file);

    let search_result: Result<Option<DataEntry>, serde_json::Error>;
    search_result =
        json_deserializer.find(|entry: &DataEntry| !entry.subscribed_to.contains(&"rust".into()));
    match search_result? {
        Some(entry) => println!(
            "Looks like {} (id: {}) doesn't like Rust... Good boy status revoked.",
            entry.name, entry.id
        ),
        None => print!("Everybody likes Rust. How cool!"),
    }
    Ok(())
}
