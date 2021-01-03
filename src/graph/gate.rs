use crate::binding::ParamBindingGet;
use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphNodeExec};

/// A graph node that executes its children only when its binding evaluates to true.
pub struct Gate<B>
where
    B: ParamBindingGet<bool>,
{
    binding: B,
}

impl<B> Gate<B>
where
    B: ParamBindingGet<bool>,
{
    /// Construct a new `Gate`
    ///
    /// # Arguments
    ///
    /// * `binding` - the binding which determines if the gate is open or closed
    pub fn new(binding: B) -> Self {
        Self { binding }
    }
}

impl<B> GraphNodeExec for Gate<B>
where
    B: ParamBindingGet<bool>,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext, children: &dyn GraphChildExec) {
        if self.binding.get() {
            children.child_exec_all(context);
        }
    }
}
