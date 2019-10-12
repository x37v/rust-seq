pub mod empty {
    use crate::event::*;
    use crate::graph::{ChildCount, GraphChildExec, GraphIndexExec};

    #[derive(Default)]
    pub struct Children;
    #[derive(Default)]
    pub struct IndexChildren;

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

    impl GraphIndexExec for IndexChildren {
        fn exec_index(&mut self, _index: usize, _context: &mut dyn EventEvalContext) {}
    }
}

/// Children build from a static mut slice
pub mod slice {
    use crate::event::*;
    use crate::graph::{
        ChildCount, GraphChildExec, GraphIndexExec, GraphNode, GraphNodeContainer,
        IndexChildContainer,
    };

    pub struct Children(&'static mut [GraphNodeContainer]);
    pub struct IndexChildren(&'static mut [IndexChildContainer]);

    impl Children {
        pub fn new(children: &'static mut [GraphNodeContainer]) -> Self {
            Self(children)
        }
    }

    impl IndexChildren {
        pub fn new(children: &'static mut [IndexChildContainer]) -> Self {
            Self(children)
        }
    }

    impl GraphChildExec for Children {
        fn child_count(&self) -> ChildCount {
            ChildCount::Some(self.0.len())
        }
        fn child_exec_range(
            &mut self,
            context: &mut dyn EventEvalContext,
            range: core::ops::Range<usize>,
        ) {
            let (_, r) = self.0.split_at_mut(range.start);
            let (r, _) = r.split_at_mut(range.end - range.start);
            for c in r {
                c.node_exec(context);
            }
        }
    }

    impl GraphIndexExec for IndexChildren {
        fn exec_index(&mut self, index: usize, context: &mut dyn EventEvalContext) {
            for c in self.0.iter_mut() {
                c.exec_index(index, context)
            }
        }
    }
}

/// Children build from a Vec
#[cfg(feature = "std")]
pub mod vec {
    use crate::event::*;
    use crate::graph::{
        ChildCount, GraphChildExec, GraphIndexExec, GraphNode, GraphNodeContainer,
        IndexChildContainer,
    };
    use std::vec::Vec;

    pub struct Children(Vec<GraphNodeContainer>);
    pub struct IndexChildren(Vec<IndexChildContainer>);

    impl Children {
        pub fn new(children: Vec<GraphNodeContainer>) -> Self {
            Self(children)
        }
    }

    impl IndexChildren {
        pub fn new(children: Vec<IndexChildContainer>) -> Self {
            Self(children)
        }
    }

    impl GraphChildExec for Children {
        fn child_count(&self) -> ChildCount {
            ChildCount::Some(self.0.len())
        }
        fn child_exec_range(
            &mut self,
            context: &mut dyn EventEvalContext,
            range: core::ops::Range<usize>,
        ) {
            let (_, r) = self.0.split_at_mut(range.start);
            let (r, _) = r.split_at_mut(range.end - range.start);
            for c in r {
                c.node_exec(context);
            }
        }
    }

    impl GraphIndexExec for IndexChildren {
        fn exec_index(&mut self, index: usize, context: &mut dyn EventEvalContext) {
            for c in self.0.iter_mut() {
                c.exec_index(index, context)
            }
        }
    }
}

/// A graph node children impl that fakes that it has infinite children.
/// It has 'index' children that get called each time a child is addressed by index.
pub mod nchild {
    use crate::event::*;
    use crate::graph::{ChildCount, GraphChildExec, GraphIndexExec, GraphNode};

    pub struct ChildWrapper<N, C>
    where
        N: GraphNode,
        C: GraphIndexExec,
    {
        child: N,
        index_children: C,
    }

    impl<N, C> ChildWrapper<N, C>
    where
        N: GraphNode,
        C: GraphIndexExec,
    {
        pub fn new(child: N, index_children: C) -> Self {
            Self {
                child,
                index_children,
            }
        }
    }

    impl<N, C> GraphChildExec for ChildWrapper<N, C>
    where
        N: GraphNode,
        C: GraphIndexExec,
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
                self.child.node_exec(context);
                self.index_children.exec_index(i, context);
            }
        }
    }
}
