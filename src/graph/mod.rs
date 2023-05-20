//! Graph items and evaluation

pub mod func;
pub mod leaf;
pub mod node;
pub mod root;
mod wrapper;

pub use wrapper::*;

use crate::event::EventEvalContext;

/// An indication of the child count for a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildCount {
    None,
    Some(usize),
    Inf,
}

/// A trait that a node, that will have children, implements.
pub trait GraphNodeExec<E, U> {
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    );
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}

/// A trait that a leaf, a node without children, implements.
pub trait GraphLeafExec<E, U> {
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, user_data: &mut U);
}

/// A trait that a node uses to execute its child nodes.
pub trait GraphChildExec<E, U> {
    /// Get the `ChildCount` value.
    fn child_count(&self) -> ChildCount;

    /// Execute children with the given index `range` and the given `context`.
    fn child_exec_range(
        &self,
        context: &mut dyn EventEvalContext<E>,
        range: core::ops::Range<usize>,
        user_data: &mut U,
    );

    /// Execute the child at the given `index` with the given `context`.
    fn child_exec(&self, context: &mut dyn EventEvalContext<E>, index: usize, user_data: &mut U) {
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
                        user_data,
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
                    user_data,
                );
            }
        }
    }

    /// Execute all children with the given `context`.
    fn child_exec_all(&self, context: &mut dyn EventEvalContext<E>, user_data: &mut U) {
        match self.child_count() {
            ChildCount::None => (),
            ChildCount::Some(i) => {
                self.child_exec_range(context, core::ops::Range { start: 0, end: i }, user_data);
            }
            ChildCount::Inf => {
                self.child_exec_range(
                    context,
                    core::ops::Range {
                        start: 0usize,
                        end: 1usize,
                    },
                    user_data,
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
pub trait GraphNode<E, U> {
    fn node_exec(&self, context: &mut dyn EventEvalContext<E>, user_data: &mut U);
}

/*
/// Automatically implement the node exec for leaf.
impl<L, E> GraphNodeExec<E> for L
where
    L: GraphLeafExec<E>,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, _children: &dyn GraphChildExec<E>) {
        self.graph_exec(context)
    }
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::None
    }
}
*/

impl<E, U> GraphChildExec<E, U> for () {
    fn child_count(&self) -> ChildCount {
        ChildCount::None
    }

    fn child_exec_range(
        &self,
        _context: &mut dyn EventEvalContext<E>,
        _range: core::ops::Range<usize>,
        _user_data: &mut U,
    ) {
        //Do nothing
    }
}
