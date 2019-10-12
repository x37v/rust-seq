use crate::binding::ParamBindingGet;
use crate::event::EventEvalContext;
use crate::graph::{ChildCount, GraphChildExec, GraphNodeExec};

/// A graph node that executes one child based on the bound index it is given.
pub struct OneHot<B>
where
    B: ParamBindingGet<usize>,
{
    binding: B,
}

impl<B> OneHot<B>
where
    B: ParamBindingGet<usize>,
{
    /// Construct a new `OneHot`
    ///
    /// # Arguments
    ///
    /// * `binding` - the binding which determines if the child to execute
    pub fn new(binding: B) -> Self {
        Self { binding }
    }
}

impl<B> GraphNodeExec for OneHot<B>
where
    B: ParamBindingGet<usize>,
{
    fn graph_exec(
        &mut self,
        context: &mut dyn EventEvalContext,
        children: &mut dyn GraphChildExec,
    ) {
        let index = self.binding.get();
        if match children.child_count() {
            ChildCount::None => false,
            ChildCount::Inf => true,
            ChildCount::Some(count) => index < count,
        } {
            children.child_exec(context, index);
        }
    }
}
