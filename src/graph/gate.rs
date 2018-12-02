use super::*;

/// A graph node that executes its children only when its binding evaluates to true.
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

impl GraphExec for Gate {
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
        if self.binding.get() {
            children.exec_all(context);
        }
        //remove self if we have no children
        children.has_children()
    }

    fn children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}
