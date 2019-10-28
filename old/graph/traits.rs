use crate::context::SchedContext;
use crate::graph::{AIndexNodeP, ANodeP, ChildCount};

pub trait GraphExec: Send {
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool;
    fn children_max(&self) -> ChildCount;
}

pub trait GraphLeafExec: Send {
    fn exec_leaf(&mut self, context: &mut dyn SchedContext);
}

pub trait GraphNodeExec: Send {
    fn exec_node(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec);
}

pub trait ChildExec {
    fn exec(&mut self, context: &mut dyn SchedContext, index: usize) -> ChildCount;
    fn exec_range(
        &mut self,
        context: &mut dyn SchedContext,
        range: core::ops::Range<usize>,
    ) -> ChildCount;
    fn exec_all(&mut self, context: &mut dyn SchedContext) -> ChildCount;
    fn count(&self) -> ChildCount;
    fn has_children(&self) -> bool;
}

pub trait GraphIndexExec: Send {
    fn exec_index(&mut self, index: usize, context: &mut dyn SchedContext);
}

pub trait ChildListT: Send {
    fn count(&self) -> usize;
    /// execute `func` on children in the range given,
    /// if func returns true, return them to the list
    fn in_range<'a>(&mut self, range: core::ops::Range<usize>, func: &'a dyn FnMut(ANodeP) -> bool);

    #[cfg(feature = "std")]
    fn push_back(&mut self, child: ANodeP);
}

pub trait IndexChildListT: Send {
    fn each<'a>(&mut self, func: &'a dyn FnMut(AIndexNodeP));

    #[cfg(feature = "std")]
    fn push_back(&mut self, child: AIndexNodeP);
}

pub trait GraphNode {
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool;
    #[cfg(feature = "std")]
    fn child_append(&mut self, child: ANodeP) -> bool;
}
