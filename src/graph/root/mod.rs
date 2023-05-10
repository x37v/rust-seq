use crate::{event::*, graph::GraphChildExec, tick::TickResched};

pub mod clock;

/// A trait for a graph root, this is executed the event schedule.
pub trait GraphRootExec<E, U> {
    fn event_eval(
        &mut self,
        context: &mut dyn EventEvalContext<E>,
        user_data: &mut U,
        children: &mut dyn GraphChildExec<E>,
    ) -> TickResched;
}

/// A wrapper that turns a `GraphRootExec` and `GraphChildExec` into an event.
pub struct GraphRootWrapper<R, C, E, U>
where
    R: GraphRootExec<E, U>,
    C: GraphChildExec<E>,
{
    pub(crate) root: R,
    pub(crate) children: C,
    pub(crate) _phantom: core::marker::PhantomData<(E, U)>,
}

impl<R, C, E, U> GraphRootWrapper<R, C, E, U>
where
    R: GraphRootExec<E, U>,
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

impl<R, C, E, U> EventEval<E, U> for GraphRootWrapper<R, C, E, U>
where
    R: GraphRootExec<E, U>,
    C: GraphChildExec<E>,
{
    fn event_eval(
        &mut self,
        context: &mut dyn EventEvalContext<E>,
        user_data: &mut U,
    ) -> TickResched {
        self.root.event_eval(context, user_data, &mut self.children)
    }
}
