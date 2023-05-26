#[cfg(feature = "with_alloc")]
pub mod boxed {
    extern crate alloc;
    use crate::{
        event::EventEvalContext,
        graph::{ChildCount, GraphChildExec, GraphNode},
    };

    /// An array of children
    pub struct GraphChildrenArray<E, U, const N: usize>(
        [alloc::boxed::Box<dyn GraphNode<E, U>>; N],
    );

    impl<E, U, const N: usize> GraphChildrenArray<E, U, N> {
        pub fn new(children: [alloc::boxed::Box<dyn GraphNode<E, U>>; N]) -> Self {
            Self(children)
        }
    }

    impl<E, U, const N: usize> GraphChildExec<E, U> for GraphChildrenArray<E, U, N>
    where
        E: Send,
        U: Send,
    {
        fn child_count(&self) -> ChildCount {
            if N == 0 {
                ChildCount::None
            } else {
                ChildCount::Some(N)
            }
        }
        fn child_exec_range(
            &self,
            context: &mut dyn EventEvalContext<E>,
            range: core::ops::Range<usize>,
            user_data: &mut U,
        ) {
            for c in self.0.iter().skip(range.start).take(range.len()) {
                c.node_exec(context, user_data);
            }
        }
    }
}

use crate::{
    event::EventEvalContext,
    graph::{ChildCount, GraphChildExec, GraphNodeSync},
};

/// An array of children
pub struct GraphChildrenArray<'a, E, U, const N: usize>([&'a dyn GraphNodeSync<E, U>; N]);

impl<'a, E, U, const N: usize> GraphChildrenArray<'a, E, U, N> {
    pub fn new(children: [&'a dyn GraphNodeSync<E, U>; N]) -> Self {
        Self(children)
    }
}

impl<'a, E, U, const N: usize> GraphChildExec<E, U> for GraphChildrenArray<'a, E, U, N>
where
    E: Send,
    U: Send,
{
    fn child_count(&self) -> ChildCount {
        if N == 0 {
            ChildCount::None
        } else {
            ChildCount::Some(N)
        }
    }
    fn child_exec_range(
        &self,
        context: &mut dyn EventEvalContext<E>,
        range: core::ops::Range<usize>,
        user_data: &mut U,
    ) {
        for c in self.0.iter().skip(range.start).take(range.len()) {
            c.node_exec(context, user_data);
        }
    }
}
