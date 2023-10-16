//! Zero-allocation deserialization of sequences located anywhere.
//!
//! This modules provides a type, [`StreamSeqDeser`] which implements [`serde::Deserialize`],
//! and can be used anywhere a sequence would be encountered. Its type parameter
//! is used to define how the sequence is aggregated in to a final value. This type
//! parameter must implement [`Aggregator`]. Though the user can provide its own implementation
//! of it, it is advised to use the provided wrapper type constructors (and their
//! associated traits) to better communicate intent and ease implementation.
//!
//! # Example
//!
//! On the following JSON
//!
//! ```JSON
//! {
//!     "api_version": "x.y.z",
//!     "result" : [
//!         {"id": 0, "name": "bob", "subscribed_to": ["rust", "knitting", "cooking"]},
//!         {"id": 1, "name": "toby 🐶", "subscribed_to": ["good-boy-lifestyle", "sticks", "tennis-balls"]},
//!         {"id": 2, "name": "alice", "subscribed_to": ["rust", "hiking", "paris"]},
//!         {"id": 3, "name": "mark", "subscribed_to": ["rust", "rugby", "doctor-who"]},
//!         {"id": 4, "name": "vera", "subscribed_to": ["rust", "mma", "philosophy"]}
//!     ]
//! }
//! ```
//!
//! We can return all channels that at leats one person (or dog) is subscribed to.
//!
//! ```rust
//! # use std::{fs::File, io::BufReader, path::PathBuf, collections::HashSet};
//!
//! #[derive(serde::Deserialize)]
//! struct DataEntry {
//!     subscribed_to: Vec<String>,
//! }
//!
//! struct Imp;
//!
//! impl serde_iter::deep::FoldImpl for Imp {
//!     type Item = DataEntry;
//!     type Acc = HashSet<String>;
//!
//!     fn init() -> Self::Acc {
//!         HashSet::new()
//!     }
//!
//!     fn f(mut acc: HashSet<String>, entry: DataEntry) -> HashSet<String> {
//!         acc.extend(entry.subscribed_to);
//!         acc
//!     }
//! }
//!
//! #[derive(serde::Deserialize)]
//! struct Data {
//!     result: serde_iter::deep::StreamSeqDeser<serde_iter::deep::Fold<Imp>>,
//! }
//!
//! fn main() -> anyhow::Result<()> {
//! #    let example_json_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "examples", "deep_data.json"]
//! #       .iter()
//! #       .collect();
//!     let buffered_file: BufReader<File> = BufReader::new(File::open(example_json_path)?);
//!
//!     let data: Data = serde_json::from_reader(buffered_file)?;
//!     let all_channels = data.result.into_inner();
//!     println!("All existing channels:");
//!     for channel in all_channels {
//!         println!("  - {channel}")
//!     }
//!     Ok(())
//! }
//! ```

use core::{
    fmt,
    marker::PhantomData,
    ops::{ControlFlow, Deref},
};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize,
};

mod fold;
pub use fold::*;

mod try_fold;
pub use try_fold::*;

mod for_each;
pub use for_each::*;

mod find;
pub use find::*;

/// The entry point for deep deserialization.
///
/// Provided with the right aggregator, it will after
/// serialization contain the aggregated value.
pub struct StreamSeqDeser<I: Aggregator> {
    value: I::Value,
}

impl<I: Aggregator> Deref for StreamSeqDeser<I> {
    type Target = I::Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<I: Aggregator> StreamSeqDeser<I> {
    /// Reference to the aggregated value
    pub fn value(&self) -> &I::Value {
        &self.value
    }

    /// Take ownership of the aggregated value
    pub fn into_inner(self) -> I::Value {
        self.value
    }
}

/// The trait on which all agregation is based.
///
/// User should often not implement this directly but rather rely on the
/// provided [implementors](#implementors) and their associated traits.
pub trait Aggregator {
    /// The accumulator type
    type Acc;
    /// The type of item deserialized from the sequence
    type Item;
    /// The type of early return value
    type Break;
    /// The final agregated type
    type Value;

    /// Initial value of the accumulator
    fn init() -> Self::Acc;

    /// The core folding function
    fn try_fold(acc: Self::Acc, item: Self::Item) -> ControlFlow<Self::Break, Self::Acc>;

    /// A finaliser obtaining the definitive aggregated value.
    ///
    /// This can be identity if `Value = ControlFlow<Self::Break, Self::Acc>`
    fn finalize(x: ControlFlow<Self::Break, Self::Acc>) -> Self::Value;
}

struct Vis<I> {
    marker: PhantomData<I>,
}

impl<'de, I: Aggregator> Visitor<'de> for Vis<I>
where
    I::Item: Deserialize<'de>,
{
    type Value = I::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut acc = I::init();
        let fin;
        'outer: {
            while let Some(value) = seq.next_element()? {
                match I::try_fold(acc, value) {
                    ControlFlow::Continue(new_acc) => acc = new_acc,
                    ControlFlow::Break(clot_break) => {
                        while seq.next_element::<I::Item>()?.is_some() {}
                        fin = ControlFlow::Break(clot_break);
                        break 'outer;
                    }
                }
            }
            fin = ControlFlow::Continue(acc)
        }
        Ok(I::finalize(fin))
    }
}

impl<'de, I: Aggregator> Deserialize<'de> for StreamSeqDeser<I>
where
    I::Item: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vis = Vis::<I> {
            marker: PhantomData,
        };
        let fin = deserializer.deserialize_seq(vis)?;
        Ok(Self { value: fin })
    }
}
