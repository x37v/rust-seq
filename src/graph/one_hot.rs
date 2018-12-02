use super::*;

/// A graph node that executes one child based on the bound index it is given.
pub struct OneHot {
    binding: BindingGetP<usize>,
}

impl OneHot {
    /// Construct a new `OneHot`
    ///
    /// # Arguments
    ///
    /// * `binding` - the binding which determines if the child to execute
    pub fn new(binding: BindingGetP<usize>) -> Self {
        Self { binding }
    }
}

impl GraphExec for OneHot {
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
        let index = self.binding.get();
        if match children.count() {
            ChildCount::None => false,
            ChildCount::Inf => true,
            ChildCount::Some(count) => index < count,
        } {
            children.exec(context, index);
        }
        //remove self if we have no children
        children.has_children()
    }

    fn children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}
