pub mod empty {
    use crate::{
        event::*,
        graph::{ChildCount, GraphChildExec},
    };

    #[derive(Default)]
    pub struct Children;

    impl GraphChildExec for Children {
        fn child_count(&self) -> ChildCount {
            ChildCount::None
        }
        fn child_exec_range(
            &mut self,
            _context: &mut dyn EventEvalContext,
            _range: core::ops::Range<usize>,
        ) {
        }
    }
}

pub mod boxed {
    extern crate alloc;
    use crate::{
        binding::ParamBindingSet,
        event::*,
        graph::{ChildCount, GraphChildExec, GraphNode, GraphNodeContainer},
    };
    use alloc::boxed::Box;
    use core::convert::From;

    pub struct Children<B>
    where
        B: ParamBindingSet<usize>,
    {
        children: Box<[GraphNodeContainer]>,
        index_binding: B,
    }

    impl Children<()> {
        /// Create children with a no-op for the index binding.
        pub fn new(children: Box<[GraphNodeContainer]>) -> Self {
            Self {
                children,
                index_binding: (),
            }
        }
    }

    impl<B> Children<B>
    where
        B: ParamBindingSet<usize>,
    {
        /// Create children, will set the child index into index_binding before executing the
        /// child.
        pub fn new_with_index(children: Box<[GraphNodeContainer]>, index_binding: B) -> Self {
            Self {
                children,
                index_binding,
            }
        }
    }

    impl<B> From<(Box<[GraphNodeContainer]>, B)> for Children<B>
    where
        B: ParamBindingSet<usize>,
    {
        fn from(children_binding: (Box<[GraphNodeContainer]>, B)) -> Self {
            Self {
                children: children_binding.0,
                index_binding: children_binding.1,
            }
        }
    }

    impl<B> GraphChildExec for Children<B>
    where
        B: ParamBindingSet<usize>,
    {
        fn child_count(&self) -> ChildCount {
            ChildCount::Some(self.children.len())
        }
        fn child_exec_range(
            &mut self,
            context: &mut dyn EventEvalContext,
            range: core::ops::Range<usize>,
        ) {
            let (_, r) = self.children.split_at_mut(range.start);
            let (r, _) = r.split_at_mut(range.end - range.start);
            for (i, c) in r.iter_mut().enumerate() {
                self.index_binding.set(i + range.start);
                c.node_exec(context);
            }
        }
    }
}

/// Children build from a Vec
#[cfg(feature = "std")]
pub mod vec {
    use crate::{
        binding::ParamBindingSet,
        event::*,
        graph::{ChildCount, GraphChildExec, GraphNode, GraphNodeContainer},
    };
    use std::vec::Vec;

    pub struct Children<B>
    where
        B: ParamBindingSet<usize>,
    {
        children: Vec<GraphNodeContainer>,
        index_binding: B,
    }

    impl Children<()> {
        pub fn new(children: Vec<GraphNodeContainer>) -> Self {
            Self {
                children,
                index_binding: (),
            }
        }
    }

    impl<B> Children<B>
    where
        B: ParamBindingSet<usize>,
    {
        pub fn new_with_index(children: Vec<GraphNodeContainer>, index_binding: B) -> Self {
            Self {
                children,
                index_binding,
            }
        }
    }

    impl<B> GraphChildExec for Children<B>
    where
        B: ParamBindingSet<usize>,
    {
        fn child_count(&self) -> ChildCount {
            ChildCount::Some(self.children.len())
        }
        fn child_exec_range(
            &mut self,
            context: &mut dyn EventEvalContext,
            range: core::ops::Range<usize>,
        ) {
            let (_, r) = self.children.split_at_mut(range.start);
            let (r, _) = r.split_at_mut(range.end - range.start);
            for (i, c) in r.iter_mut().enumerate() {
                self.index_binding.set(i + range.start);
                c.node_exec(context);
            }
        }
    }
}

/// A graph node children impl that fakes that it has infinite children.
pub mod nchild {
    use crate::{
        binding::ParamBindingSet,
        event::*,
        graph::{ChildCount, GraphChildExec, GraphNode},
    };

    pub struct ChildWrapper<N, B>
    where
        N: GraphNode,
        B: ParamBindingSet<usize>,
    {
        child: N,
        index_binding: B,
    }

    impl<N> ChildWrapper<N, ()>
    where
        N: GraphNode,
    {
        pub fn new(child: N) -> Self {
            Self {
                child,
                index_binding: (),
            }
        }
    }

    impl<N, B> ChildWrapper<N, B>
    where
        N: GraphNode,
        B: ParamBindingSet<usize>,
    {
        pub fn new_with_index(child: N, index_binding: B) -> Self {
            Self {
                child,
                index_binding,
            }
        }
    }

    impl<N, B> GraphChildExec for ChildWrapper<N, B>
    where
        N: GraphNode,
        B: ParamBindingSet<usize>,
    {
        fn child_count(&self) -> ChildCount {
            ChildCount::Inf
        }
        fn child_exec_range(
            &mut self,
            context: &mut dyn EventEvalContext,
            range: core::ops::Range<usize>,
        ) {
            for i in range {
                self.index_binding.set(i);
                self.child.node_exec(context);
            }
        }
    }
}
