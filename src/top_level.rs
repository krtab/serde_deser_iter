//! Zero-allocation deserialization of sequences located at the top-level of the data file
//!
//! This modules provides only one trait, [`DeserializerExt`] which extends [`serde::Deserializer`]
//! with methods similar to those used to aggregate data from an iterator
//! (`fold`, `for_each`, `find`).
//!
//! To use it, simply `use serde_iter::top_level::DeserializerExt` and use the appropriated method
//! from [`DeserializerExt`].
//!
//! # Example
//!
//! Given the following JSON:
//!
//! ```json
//! [
//!     {"id": 0, "name": "bob", "subscribed_to": ["rust", "knitting", "cooking"]},
//!     {"id": 1, "name": "toby 🐶", "subscribed_to": ["sticks", "tennis-balls"]},
//!     {"id": 2, "name": "alice", "subscribed_to": ["rust", "hiking", "paris"]},
//!     {"id": 3, "name": "mark", "subscribed_to": ["rust", "rugby", "doctor-who"]},
//!     {"id": 4, "name": "vera", "subscribed_to": ["rust", "mma", "philosophy"]}
//! ]
//! ```
//! we can process it without allocating a 5-sized vector of items as follow:
//!
//! ```
//! use serde_iter::top_level::DeserializerExt;
//! # use std::{fs::File, io::BufReader, path::PathBuf};
//!
//! #[derive(serde::Deserialize)]
//! struct DataEntry {
//!     id: u32,
//!     name: String,
//!     subscribed_to: Vec<String>,
//! }
//!
//! fn main() -> anyhow::Result<()> {
//! #    let example_json_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "examples", "data.json"]
//! #       .iter()
//! #       .collect();
//!     let buffered_file: BufReader<File> = BufReader::new(File::open(example_json_path)?);
//!     let mut json_deserializer = serde_json::Deserializer::from_reader(buffered_file);
//!     
//!     let search_result: Result<Option<DataEntry>, serde_json::Error>;
//!     search_result =
//!         json_deserializer.find(|entry: &DataEntry| !entry.subscribed_to.contains(&"rust".into()));
//!     match search_result? {
//!         Some(entry) => println!(
//!             "Looks like {} (id: {}) doesn't like Rust... Good boy status revoked.",
//!             entry.name, entry.id
//!         ),
//!         None => print!("Everybody likes Rust. How cool!"),
//!     }
//!     Ok(())
//! }
//! ```

use core::{convert::Infallible, fmt, marker::PhantomData, ops::ControlFlow};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};

struct DeserTryFolder<Acc, Item, Err, F> {
    #[allow(clippy::type_complexity)]
    marker: PhantomData<fn(Acc, Item) -> ControlFlow<Err, Acc>>,
    init: Acc,
    f: F,
}

impl<Acc, Item, Err, F> DeserTryFolder<Acc, Item, Err, F> {
    pub fn new(init: Acc, f: F) -> Self {
        Self {
            marker: PhantomData,
            f,
            init,
        }
    }
}

struct Wrapper<T>(T);

impl<'de, Acc, Item, Err, F> Visitor<'de> for Wrapper<DeserTryFolder<Acc, Item, Err, F>>
where
    F: FnMut(Acc, Item) -> ControlFlow<Err, Acc>,
    Item: Deserialize<'de>,
{
    type Value = ControlFlow<Err, Acc>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut acc = self.0.init;
        while let Some(value) = seq.next_element()? {
            match (self.0.f)(acc, value) {
                ControlFlow::Continue(new_acc) => acc = new_acc,
                ControlFlow::Break(clot_break) => {
                    while seq.next_element::<Item>()?.is_some() {}
                    return Ok(ControlFlow::Break(clot_break));
                }
            }
        }
        Ok(ControlFlow::Continue(acc))
    }
}

fn lift_infallible<T>(val: T) -> ControlFlow<Infallible, T> {
    ControlFlow::Continue(val)
}

/// The workhorse of this module.
///
/// See module-level [doc](`crate::top_level`) for a broad level explanation.
pub trait DeserializerExt<'de, Item>: Deserializer<'de>
where
    Item: Deserialize<'de>,
{
    /// Aggregate all items from the sequence using a fallible/early-returning function.
    ///
    /// **Caution:** The early return [caveat](../index.html#early-returns) applies.
    fn try_fold<Acc, Err, F>(self, init: Acc, f: F) -> Result<ControlFlow<Err, Acc>, Self::Error>
    where
        F: FnMut(Acc, Item) -> ControlFlow<Err, Acc>,
    {
        let folder = DeserTryFolder::new(init, f);
        self.deserialize_seq(Wrapper(folder))
    }

    /// Aggregate all items from the sequence. If the function may fail or needs to return early
    /// use [try_fold](`DeserializerExt::try_fold`).
    fn fold<Acc, F>(self, init: Acc, mut f: F) -> Result<Acc, Self::Error>
    where
        F: FnMut(Acc, Item) -> Acc,
    {
        match self.try_fold(init, |acc, item| lift_infallible(f(acc, item))) {
            Ok(ControlFlow::Break(_infallible)) => unreachable!(),
            Ok(ControlFlow::Continue(res)) => Ok(res),
            Err(e) => Err(e),
        }
    }

    /// Run a cloture with side-effects on all items of the sequence.
    fn for_each<F>(self, mut f: F) -> Result<(), Self::Error>
    where
        F: FnMut(Item),
    {
        self.fold((), |(), item| f(item))
    }

    /// Find an item matching the predicate
    ///
    /// **Caution:** The early return [caveat](../index.html#early-returns) applies.
    fn find<F>(self, mut f: F) -> Result<Option<Item>, Self::Error>
    where
        F: for<'a> FnMut(&'a Item) -> bool,
    {
        let fold_res = self.try_fold((), |(), item| {
            if f(&item) {
                ControlFlow::Break(item)
            } else {
                ControlFlow::Continue(())
            }
        });
        let res = match fold_res? {
            ControlFlow::Continue(()) => None,
            ControlFlow::Break(item) => Some(item),
        };
        Ok(res)
    }
}

impl<'de, Item, D> DeserializerExt<'de, Item> for D
where
    D: Deserializer<'de>,
    Item: Deserialize<'de>,
{
}
