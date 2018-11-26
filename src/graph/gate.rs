use super::*;

pub struct Gate {
    binding: BindingGetP<bool>,
}

impl Gate {
    pub fn new_p(binding: BindingGetP<bool>) -> Box<Self> {
        Box::new(Self { binding })
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
