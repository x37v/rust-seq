use super::*;
use crate::binding::ParamBindingGet;

/// A graph node that executes its children only when its binding evaluates to true.
#[derive(GraphNode)]
pub struct Gate<B: ParamBindingGet<bool>> {
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
    fn exec_node(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) {
        if self.binding.get() {
            children.exec_all(context);
        }
    }
}
