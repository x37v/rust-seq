use crate::{event::*, graph::GraphChildExec, tick::TickResched};

use core::cmp::Ordering;

pub mod clock;

/// A trait for a graph root, this is executed the event schedule.
pub trait GraphRootExec<E>: Send {
    fn event_eval(
        &mut self,
        context: &mut dyn EventEvalContext<E>,
        children: &mut dyn GraphChildExec<E>,
    ) -> TickResched;
}

/// A wrapper that turns a `GraphRootExec` and `GraphChildExec` into an event.
pub struct GraphRootWrapper<R, C, E>
where
    R: GraphRootExec<E>,
    C: GraphChildExec<E>,
{
    pub(crate) root: R,
    pub(crate) children: C,
    pub(crate) _phantom: core::marker::PhantomData<E>,
}

impl<R, C, E> GraphRootWrapper<R, C, E>
where
    R: GraphRootExec<E>,
    C: GraphChildExec<E>,
{
    pub fn new(root: R, children: C) -> Self {
        Self {
            root,
            children,
            _phantom: Default::default(),
        }
    }
}

impl<R, C, E> EventEval<E> for GraphRootWrapper<R, C, E>
where
    R: GraphRootExec<E>,
    C: GraphChildExec<E>,
    E: Send,
{
    fn event_eval(&mut self, context: &mut dyn EventEvalContext<E>) -> TickResched {
        self.root.event_eval(context, &mut self.children)
    }
}

impl<R, C, E> Ord for GraphRootWrapper<R, C, E>
where
    R: GraphRootExec<E>,
    C: GraphChildExec<E>,
    E: Send,
{
    fn cmp(&self, other: &Self) -> Ordering {
        let left = &self.root as *const R;
        let right = &other.root as *const R;
        left.cmp(&right)
    }
}

impl<R, C, E> PartialOrd for GraphRootWrapper<R, C, E>
where
    R: GraphRootExec<E>,
    C: GraphChildExec<E>,
    E: Send,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<R, C, E> PartialEq for GraphRootWrapper<R, C, E>
where
    R: GraphRootExec<E>,
    C: GraphChildExec<E>,
    E: Send,
{
    fn eq(&self, _other: &Self) -> bool {
        false //box, never equal
    }
}

impl<R, C, E> Eq for GraphRootWrapper<R, C, E>
where
    R: GraphRootExec<E>,
    C: GraphChildExec<E>,
    E: Send,
{
}
