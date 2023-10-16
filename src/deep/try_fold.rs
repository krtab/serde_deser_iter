use core::{marker::PhantomData, ops::ControlFlow};

use super::Aggregator;

/// A wrapper for a fallible/early-returning folding aggregator
pub struct TryFold<I> {
    marker: PhantomData<I>,
}

/// Functions for fallible folding aggregation
pub trait TryFoldAggregator {
    /// The Item deserialized from the sequences
    type Item;
    /// The accumulator type
    type Acc;
    /// The early return type
    type Break;

    /// Initial value of the accumulator
    fn init() -> Self::Acc;

    /// Core folding function
    fn f(acc: Self::Acc, item: Self::Item) -> ControlFlow<Self::Break, Self::Acc>;
}

impl<I> Aggregator for TryFold<I>
where
    I: TryFoldAggregator,
{
    type Acc = I::Acc;

    type Item = I::Item;

    type Break = I::Break;

    type Value = ControlFlow<I::Break, I::Acc>;

    fn init() -> Self::Acc {
        I::init()
    }

    fn try_fold(acc: Self::Acc, item: Self::Item) -> ControlFlow<Self::Break, Self::Acc> {
        I::f(acc, item)
    }

    fn finalize(x: ControlFlow<Self::Break, Self::Acc>) -> Self::Value {
        x
    }
}
