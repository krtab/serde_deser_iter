use serde_deser_iter::top_level::DeserializerExt;
use std::{fs::File, io::BufReader, path::PathBuf};

#[derive(serde::Deserialize)]
struct DataEntry {
    year: u32,
}

fn main() -> anyhow::Result<()> {
    let example_json_path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "examples",
        "top_level_movies.json",
    ]
    .iter()
    .collect();
    let buffered_file: BufReader<File> = BufReader::new(File::open(example_json_path)?);
    let mut json_deserializer = serde_json::Deserializer::from_reader(buffered_file);

    let mut counts = vec![0; 130];
    json_deserializer.for_each(|entry: DataEntry| {
        let idx = (entry.year - 1900) as usize;
        counts[idx] += 1;
    })?;
    for (idx, v) in counts.into_iter().enumerate() {
        println!("{year}: {v}", year = 1900 + idx);
    }
    Ok(())
}
