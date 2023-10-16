use core::{marker::PhantomData, ops::ControlFlow};

use super::Aggregator;

/// A wrapper for a searching aggregator
pub struct Find<I> {
    marker: PhantomData<I>,
}

/// Functions for searching aggregation
pub trait FindAggregator {
    /// The Item deserialized from the sequences
    type Item;

    /// Core searching function
    fn f(item: &Self::Item) -> bool;
}

impl<I> Aggregator for Find<I>
where
    I: FindAggregator,
{
    type Acc = ();

    type Item = I::Item;

    type Break = I::Item;

    type Value = Option<I::Item>;

    #[allow(clippy::unused_unit)]
    fn init() -> Self::Acc {
        ()
    }

    fn try_fold((): Self::Acc, item: Self::Item) -> ControlFlow<Self::Break, Self::Acc> {
        if I::f(&item) {
            ControlFlow::Break(item)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn finalize(f: ControlFlow<Self::Break, Self::Acc>) -> Self::Value {
        match f {
            ControlFlow::Continue(()) => None,
            ControlFlow::Break(item) => Some(item),
        }
    }
}
