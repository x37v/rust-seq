use crate::context::SchedContext;
use crate::graph::ChildCount;

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
        range: std::ops::Range<usize>,
    ) -> ChildCount;
    fn exec_all(&mut self, context: &mut dyn SchedContext) -> ChildCount;
    fn count(&self) -> ChildCount;
    fn has_children(&self) -> bool;
}

pub trait GraphIndexExec: Send {
    fn exec_index(&mut self, index: usize, context: &mut dyn SchedContext);
}

cfg_if! {
if #[cfg(feature = "with_std")] {
    use crate::graph::AChildP;

    pub trait GraphNode {
        fn exec(&mut self, context: &mut dyn SchedContext) -> bool;
        fn child_append(&mut self, child: AChildP) -> bool;
    }
} else {
    pub trait GraphNode {
        fn exec(&mut self, context: &mut dyn SchedContext) -> bool;
    }
}
}
