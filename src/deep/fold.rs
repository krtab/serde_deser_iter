use core::{convert::Infallible, marker::PhantomData, ops::ControlFlow};

use super::Aggregator;

/// A wrapper for a folding aggregator
pub struct Fold<I> {
    marker: PhantomData<I>,
}

/// Functions for folding aggregation
pub trait FoldAggregator {
    /// The Item deserialized from the sequences
    type Item;
    /// The accumulator type
    type Acc;

    /// Initial value of the accumulator
    fn init() -> Self::Acc;

    /// Core folding function
    fn f(acc: Self::Acc, item: Self::Item) -> Self::Acc;
}

impl<I> Aggregator for Fold<I>
where
    I: FoldAggregator,
{
    type Acc = I::Acc;

    type Item = I::Item;

    type Break = Infallible;

    type Value = I::Acc;

    fn init() -> Self::Acc {
        I::init()
    }

    fn try_fold(acc: Self::Acc, item: Self::Item) -> ControlFlow<Self::Break, Self::Acc> {
        ControlFlow::Continue(I::f(acc, item))
    }

    fn finalize(f: ControlFlow<Self::Break, Self::Acc>) -> Self::Value {
        match f {
            ControlFlow::Continue(acc) => acc,
            ControlFlow::Break(_) => unreachable!(),
        }
    }
}
