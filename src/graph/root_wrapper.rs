use crate::{
    event::*,
    graph::{GraphChildExec, GraphRootExec},
    tick::TickResched,
};

pub struct GraphRootWrapper<E, C>
where
    E: GraphRootExec,
    C: GraphChildExec,
{
    exec: E,
    children: C,
}

impl<E, C> GraphRootWrapper<E, C>
where
    E: GraphRootExec,
    C: GraphChildExec,
{
    pub fn new(exec: E, children: C) -> Self {
        Self { exec, children }
    }
}

impl<E, C> EventEval for GraphRootWrapper<E, C>
where
    E: GraphRootExec,
    C: GraphChildExec,
{
    fn event_eval(&mut self, context: &mut dyn EventEvalContext) -> TickResched {
        self.exec.event_eval(context, &mut self.children)
    }
}
