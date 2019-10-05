use crate::event::*;
use crate::graph::ChildCount;

pub trait GraphChildExec {
    fn child_count(&self) -> ChildCount;
    fn child_exec_range(
        &mut self,
        context: &mut dyn EventEvalContext,
        range: core::ops::Range<usize>,
    );
    fn child_exec(&mut self, context: &mut dyn EventEvalContext, index: usize) {
        match self.child_count() {
            ChildCount::None => (),
            ChildCount::Some(i) => {
                if i > index {
                    self.child_exec_range(
                        context,
                        core::ops::Range {
                            start: index,
                            end: index + 1,
                        },
                    );
                }
            }
            ChildCount::Inf => {
                self.child_exec_range(
                    context,
                    core::ops::Range {
                        start: 0usize,
                        end: 1usize,
                    },
                );
            }
        }
    }
    fn child_exec_all(&mut self, context: &mut dyn EventEvalContext) {
        match self.child_count() {
            ChildCount::None => (),
            ChildCount::Some(i) => {
                self.child_exec_range(context, core::ops::Range { start: 0, end: i });
            }
            ChildCount::Inf => {
                self.child_exec_range(
                    context,
                    core::ops::Range {
                        start: 0usize,
                        end: 1usize,
                    },
                );
            }
        }
    }
    fn child_have(&self) -> bool {
        match self.child_count() {
            ChildCount::None => false,
            ChildCount::Some(_) | ChildCount::Inf => true,
        }
    }
}

/// A trait that executes a node and, if appropriate, its children.
///
/// This would be implemented for a wrapper a round a GraphNodeExec or GraphLeafExec
/// and potentially some children.
pub trait GraphNode: Send {
    fn exec(&mut self, context: &mut dyn EventEvalContext);
}

/// A trait that a node, that will have children, implements
pub trait GraphNodeExec: Send {
    fn graph_exec(&mut self, context: &mut dyn EventEvalContext, children: &mut dyn GraphChildExec);
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}

/// A trait that a leaf, a node without children, implements
pub trait GraphLeafExec: Send {
    fn graph_exec_leaf(&mut self, context: &mut dyn EventEvalContext);
}

/// automatically implement the node exec for leafs
impl<T> GraphNodeExec for T
where
    T: GraphLeafExec,
{
    fn graph_exec(
        &mut self,
        context: &mut dyn EventEvalContext,
        _children: &mut dyn GraphChildExec,
    ) {
        self.graph_exec_leaf(context)
    }
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::None
    }
}

/*
impl GraphChildExec for &'static [&dyn GraphNodeExec] {
    fn child_count(&self) -> ChildCount {
        if self.len() == 0 {
            ChildCount::None
        } else {
            ChildCount::Some(self.len())
        }
    }
    fn child_exec_range(
        &mut self,
        context: &mut dyn EventEvalContext,
        range: core::ops::Range<usize>,
    ) {
        let l = self.len();
        if l > 0 {
            let r = Range {
                start: core::cmp::min(l - 1, range.start),
                end: core::cmp::min(l, range.end),
            };
        }
    }
}
*/
