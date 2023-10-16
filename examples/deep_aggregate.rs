use std::{collections::HashSet, fs::File, io::BufReader, path::PathBuf};

#[derive(serde::Deserialize)]
struct DataEntry {
    subscribed_to: Vec<String>,
}

struct Imp;

impl serde_deser_iter::deep::FoldAggregator for Imp {
    type Item = DataEntry;
    type Acc = HashSet<String>;

    fn init() -> Self::Acc {
        HashSet::new()
    }

    fn f(mut acc: HashSet<String>, entry: DataEntry) -> HashSet<String> {
        acc.extend(entry.subscribed_to);
        acc
    }
}

#[derive(serde::Deserialize)]
struct Data {
    result: serde_deser_iter::deep::StreamSeqDeser<serde_deser_iter::deep::Fold<Imp>>,
}

fn main() -> anyhow::Result<()> {
    let example_json_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "examples", "deep_data.json"]
        .iter()
        .collect();
    let buffered_file: BufReader<File> = BufReader::new(File::open(example_json_path)?);

    let data: Data = serde_json::from_reader(buffered_file)?;
    let all_channels = data.result.into_inner();
    println!("All existing channels:");
    for channel in all_channels {
        println!("  - {channel}")
    }
    Ok(())
}
