use crate::event::*;

pub mod clock_ratio;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ChildCount {
    None,
    Some(usize),
    Inf,
}

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

pub trait GraphNodeExec: Send {
    fn graph_exec(&mut self, context: &mut dyn EventEvalContext, children: &mut dyn GraphChildExec);
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}

/// A graph node with no children, will never have children
pub trait GraphLeafExec: Send {
    fn graph_exec_leaf(&mut self, context: &mut dyn EventEvalContext);
}

/// automatically implement the node exec for leafs
impl<T> GraphNodeExec for T
where
    T: GraphLeafExec,
{
    fn graph_exec(&mut self, context: &mut dyn EventEvalContext, _children: &mut dyn GraphChildExec) {
        self.graph_exec_leaf(context)
    }
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::None
    }
}
