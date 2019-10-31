use crate::binding::ParamBindingSet;
use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphNodeExec};

/// A graph node that stores the latest context or absolute tick
/// it is given in a binding.
pub enum TickRecord<B>
where
    B: ParamBindingSet<usize>,
{
    Absolute(B),
    Context(B),
}

impl<B> GraphNodeExec for TickRecord<B>
where
    B: ParamBindingSet<usize>,
{
    fn graph_exec(
        &mut self,
        context: &mut dyn EventEvalContext,
        children: &mut dyn GraphChildExec,
    ) {
        match self {
            Self::Absolute(b) => b.set(context.tick_now()),
            Self::Context(b) => b.set(context.context_tick_now()),
        }
        children.child_exec_all(context);
    }
}
