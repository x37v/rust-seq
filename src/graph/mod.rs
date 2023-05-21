//! Graph items and evaluation

pub mod children;
pub mod func;
pub mod leaf;
pub mod node;
pub mod root;

mod wrapper;

pub use wrapper::*;

#[cfg(feature = "with_alloc")]
extern crate alloc;

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

impl<T, E, U> GraphNodeExec<E, U> for &T
where
    T: GraphNodeExec<E, U>,
{
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    ) {
        T::graph_exec(self, context, children, user_data)
    }
    fn graph_children_max(&self) -> ChildCount {
        T::graph_children_max(self)
    }
}

impl<T, E, U> GraphChildExec<E, U> for &T
where
    T: GraphChildExec<E, U>,
{
    fn child_count(&self) -> ChildCount {
        T::child_count(self)
    }

    fn child_exec_range(
        &self,
        context: &mut dyn EventEvalContext<E>,
        range: core::ops::Range<usize>,
        user_data: &mut U,
    ) {
        T::child_exec_range(self, context, range, user_data)
    }
}

impl<T, E, U> GraphNode<E, U> for &T
where
    T: GraphNode<E, U>,
{
    fn node_exec(&self, context: &mut dyn EventEvalContext<E>, user_data: &mut U) {
        T::node_exec(self, context, user_data)
    }
}

#[cfg(feature = "with_alloc")]
impl<T, E, U> GraphNodeExec<E, U> for alloc::boxed::Box<T>
where
    T: GraphNodeExec<E, U>,
{
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    ) {
        T::graph_exec(self, context, children, user_data)
    }
    fn graph_children_max(&self) -> ChildCount {
        T::graph_children_max(self)
    }
}

#[cfg(feature = "with_alloc")]
impl<T, E, U> GraphChildExec<E, U> for alloc::boxed::Box<T>
where
    T: GraphChildExec<E, U>,
{
    fn child_count(&self) -> ChildCount {
        T::child_count(self)
    }

    fn child_exec_range(
        &self,
        context: &mut dyn EventEvalContext<E>,
        range: core::ops::Range<usize>,
        user_data: &mut U,
    ) {
        T::child_exec_range(self, context, range, user_data)
    }
}

#[cfg(feature = "with_alloc")]
impl<T, E, U> GraphNode<E, U> for alloc::boxed::Box<T>
where
    T: GraphNode<E, U>,
{
    fn node_exec(&self, context: &mut dyn EventEvalContext<E>, user_data: &mut U) {
        T::node_exec(self, context, user_data)
    }
}

//dummy graph node, useful while spinning up graph
impl<E, U> GraphNodeExec<E, U> for () {
    fn graph_exec(
        &self,
        _context: &mut dyn EventEvalContext<E>,
        _children: &dyn GraphChildExec<E, U>,
        _user_data: &mut U,
    ) {
    }
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}

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
