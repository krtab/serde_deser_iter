use core::{convert::Infallible, marker::PhantomData, ops::ControlFlow};

use super::Aggregator;

/// A wrapper to apply a function to each element
pub struct ForEach<I> {
    marker: PhantomData<I>,
}

/// Functions for iteration
pub trait ForEachAggregator {
    /// The Item deserialized from the sequences
    type Item;

    /// Core function
    fn f(item: Self::Item);
}

impl<I> Aggregator for ForEach<I>
where
    I: ForEachAggregator,
{
    type Acc = ();

    type Item = I::Item;

    type Break = Infallible;

    type Value = ();

    #[allow(clippy::unused_unit)]
    fn init() -> Self::Acc {
        ()
    }

    fn try_fold((): Self::Acc, item: Self::Item) -> ControlFlow<Self::Break, Self::Acc> {
        I::f(item);
        ControlFlow::Continue(())
    }

    fn finalize(f: ControlFlow<Self::Break, Self::Acc>) -> Self::Value {
        match f {
            ControlFlow::Continue(acc) => acc,
            ControlFlow::Break(_) => unreachable!(),
        }
    }
}
