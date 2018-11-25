use binding::BindingGetP;
use context::SchedContext;
use graph::{ChildCount, ChildExec, GraphExec};
use rand::prelude::*;

pub struct ProbabilityGate {
    probability: BindingGetP<f32>,
}

impl ProbabilityGate {
    pub fn new_p(probability: BindingGetP<f32>) -> Box<Self> {
        Box::new(Self { probability })
    }
}

impl GraphExec for ProbabilityGate {
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
        let v = self.probability.get();
        if v >= thread_rng().gen() {
            //XXX should we specify the Rng and store it in the struct?
            children.exec_all(context);
        }
        //remove self if we have no children
        children.has_children()
    }

    fn children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}
