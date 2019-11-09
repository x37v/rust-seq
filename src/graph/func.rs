use crate::event::EventEvalContext;
use crate::graph::{GraphChildExec, GraphLeafExec, GraphNodeExec};

pub struct LeafFunc<F> {
    func: F,
}

pub struct NodeFunc<F> {
    func: F,
}

impl<F> LeafFunc<F>
where
    F: Fn(&mut dyn EventEvalContext) + Send,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> GraphLeafExec for LeafFunc<F>
where
    F: Fn(&mut dyn EventEvalContext) + Send,
{
    fn graph_exec(&mut self, context: &mut dyn EventEvalContext) {
        (self.func)(context);
    }
}

impl<F> NodeFunc<F>
where
    F: Fn(&mut dyn EventEvalContext, &mut dyn GraphChildExec) + Send,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> GraphNodeExec for NodeFunc<F>
where
    F: Fn(&mut dyn EventEvalContext, &mut dyn GraphChildExec) + Send,
{
    fn graph_exec(
        &mut self,
        context: &mut dyn EventEvalContext,
        children: &mut dyn GraphChildExec,
    ) {
        (self.func)(context, children);
    }
}
