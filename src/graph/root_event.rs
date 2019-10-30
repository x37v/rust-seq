use crate::binding::ParamBindingGet;
use crate::event::{EventEval, EventEvalContext};
use crate::graph::GraphNode;
use crate::tick::TickResched;

/// A event_eval schedulable item that holds and executes a graph tree root.
/// Holds a binding to indicate if/when to reschedule.
pub struct RootEvent<T, N>
where
    T: GraphNode,
    N: ParamBindingGet<TickResched>,
{
    root: T,
    next_binding: N,
}

impl<T, N> RootEvent<T, N>
where
    T: GraphNode,
    N: ParamBindingGet<TickResched>,
{
    /// Construct a new `RootEvent`
    ///
    /// # Arguments
    ///
    /// * `root` - the graph root to evaluate
    /// * `next_binding` - the binding which determines if/when to reschedule.
    pub fn new(root: T, next_binding: N) -> Self {
        Self { root, next_binding }
    }
}

impl<T, N> EventEval for RootEvent<T, N>
where
    T: GraphNode,
    N: ParamBindingGet<TickResched>,
{
    fn event_eval(&mut self, context: &mut dyn EventEvalContext) -> TickResched {
        self.root.node_exec(context);
        self.next_binding.get()
    }
}
