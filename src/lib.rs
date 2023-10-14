#![no_std]
use core::convert::Infallible;
use core::marker::PhantomData;
use core::{fmt, ops::ControlFlow};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};

struct DeserTryFolder<Acc, Item, Err, F> {
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
                ControlFlow::Break(clot_break) => return Ok(ControlFlow::Break(clot_break)),
            }
        }
        Ok(ControlFlow::Continue(acc))
    }
}

fn lift_infallible<T>(val: T) -> ControlFlow<Infallible, T> {
    ControlFlow::Continue(val)
}

pub trait DeserializerExt<'de, Item>: Deserializer<'de>
where
    Item: Deserialize<'de>,
{
    fn try_fold<Acc, Err, F>(self, init: Acc, f: F) -> Result<ControlFlow<Err, Acc>, Self::Error>
    where
        F: FnMut(Acc, Item) -> ControlFlow<Err, Acc>,
    {
        let folder = DeserTryFolder::new(init, f);
        self.deserialize_seq(Wrapper(folder))
    }

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

    fn for_each<F>(self, mut f: F) -> Result<(), Self::Error>
    where
        F: FnMut(Item),
    {
        self.fold((), |(), item| f(item))
    }

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
