use super::*;

/// A graph node that executes its children only when its binding evaluates to true.
#[derive(GraphNode)]
pub struct Gate {
    binding: BindingGetP<bool>,
}

impl Gate {
    /// Construct a new `Gate`
    ///
    /// # Arguments
    ///
    /// * `binding` - the binding which determines if the gate is open or closed
    pub fn new(binding: BindingGetP<bool>) -> Self {
        Self { binding }
    }
}

impl GraphNodeExec for Gate {
    fn exec_node(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) {
        if self.binding.get() {
            children.exec_all(context);
        }
    }
}
