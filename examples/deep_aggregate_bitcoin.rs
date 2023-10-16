use std::{fs::File, io::BufReader, path::PathBuf};

struct Imp;

impl serde_iter::deep::FoldAggregator for Imp {
    type Item = u64;
    type Acc = u64;

    fn init() -> u64 {
        0
    }

    fn f(acc: u64, item: u64) -> u64 {
        acc.max(item)
    }
}

#[derive(serde::Deserialize)]
struct BitCoin {
    #[serde(rename = "txIndexes")]
    tx_indexes: serde_iter::deep::StreamSeqDeser<serde_iter::deep::Fold<Imp>>,
}

fn main() -> anyhow::Result<()> {
    let example_json_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "examples", "deep_bitcoin.json"]
        .iter()
        .collect();
    let buffered_file: BufReader<File> = BufReader::new(File::open(example_json_path)?);

    let content: BitCoin = serde_json::from_reader(buffered_file)?;
    println!("Max transaction: {}", content.tx_indexes.value());
    Ok(())
}
