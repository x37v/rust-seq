use crate::{event::*, graph::GraphChildExec, tick::TickResched};

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
    root: R,
    children: C,
    _phantom: core::marker::PhantomData<E>,
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
