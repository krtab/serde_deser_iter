Allow aggregating data from large serialized sequences (think 1GB Json/Yaml/other with one top-level array containing `Item`s). Avoids deserializing and storing the entire sequence by providing methods similar to the one defined on `Iterator`.

Extends `Deserialize` with:

```rust
pub trait DeserializerExt<'de, Item>: Deserializer<'de>
where
    Item: Deserialize<'de>,
{
    fn try_fold<Acc, Err, F>(
        self,
        init: Acc,
        f: F
    ) -> Result<ControlFlow<Err, Acc>, Self::Error>
       where F: FnMut(Acc, Item) -> ControlFlow<Err, Acc>;

    fn fold<Acc, F>(self, init: Acc, f: F) -> Result<Acc, Self::Error>
       where F: FnMut(Acc, Item) -> Acc;

    fn for_each<F>(self, f: F) -> Result<(), Self::Error>
       where F: FnMut(Item);

    fn find<F>(self, f: F) -> Result<Option<Item>, Self::Error>
       where F: for<'a> FnMut(&'a Item) -> bool;
}
```