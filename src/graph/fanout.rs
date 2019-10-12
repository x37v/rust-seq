use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphNodeExec};

/// Simply evaluates all children when called
pub struct FanOut;

impl FanOut {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FanOut {
    fn default() -> Self {
        Self
    }
}

impl GraphNodeExec for FanOut {
    fn graph_exec(
        &mut self,
        context: &mut dyn EventEvalContext,
        children: &mut dyn GraphChildExec,
    ) {
        children.child_exec_all(context);
    }
}
