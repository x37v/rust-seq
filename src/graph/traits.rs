use crate::{event::*, graph::ChildCount, tick::TickResched};

/// A trait that a node uses to execute its child nodes.
pub trait GraphChildExec: Send {
    /// Get the `ChildCount` value.
    fn child_count(&self) -> ChildCount;

    /// Execute children with the given index `range` and the given `context`.
    fn child_exec_range(&self, context: &mut dyn EventEvalContext, range: core::ops::Range<usize>);

    /// Execute the child at the given `index` with the given `context`.
    fn child_exec(&self, context: &mut dyn EventEvalContext, index: usize) {
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
                        start: index,
                        end: index + 1,
                    },
                );
            }
        }
    }

    /// Execute all children with the given `context`.
    fn child_exec_all(&self, context: &mut dyn EventEvalContext) {
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

    /// Are there any children.
    fn child_any(&self) -> bool {
        match self.child_count() {
            ChildCount::None => false,
            ChildCount::Some(_) | ChildCount::Inf => true,
        }
    }
}

/// A trait for a node that wraps something that implements GraphNodeExec and GraphChildExec
pub trait GraphNode: Send {
    fn node_exec(&self, context: &mut dyn EventEvalContext);
}

/// A trait for a graph root, this is executed the event schedule.
pub trait GraphRootExec: Send {
    fn event_eval(
        &mut self,
        context: &mut dyn EventEvalContext,
        children: &mut dyn GraphChildExec,
    ) -> TickResched;
}

/// A trait that a node, that will have children, implements.
pub trait GraphNodeExec: Send {
    fn graph_exec(&self, context: &mut dyn EventEvalContext, children: &dyn GraphChildExec);
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}

/// A trait that a leaf, a node without children, implements.
pub trait GraphLeafExec: Send {
    fn graph_exec(&self, context: &mut dyn EventEvalContext);
}

/// Automatically implement the node exec for leaf.
impl<T> GraphNodeExec for T
where
    T: GraphLeafExec,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext, _children: &dyn GraphChildExec) {
        self.graph_exec(context)
    }
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::None
    }
}

impl<T> GraphNodeExec for &'static spin::Mutex<T>
where
    T: GraphNodeExec,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext, children: &dyn GraphChildExec) {
        self.lock().graph_exec(context, children)
    }

    fn graph_children_max(&self) -> ChildCount {
        self.lock().graph_children_max()
    }
}

impl<T> GraphNode for &'static spin::Mutex<T>
where
    T: GraphNode,
{
    fn node_exec(&self, context: &mut dyn EventEvalContext) {
        self.lock().node_exec(context)
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
