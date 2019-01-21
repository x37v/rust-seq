use super::*;
use crate::ptr::UniqPtr;

pub struct GraphNodeWrapper {
    exec: UniqPtr<dyn GraphExec>,
    children: ChildList,
}

pub struct NChildGraphNodeWrapper {
    exec: UniqPtr<dyn GraphExec>,
    children: ChildList,
    index_children: IndexChildList,
}

impl GraphNode for GraphNodeWrapper {
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool {
        let mut children = Children::new(&mut self.children);
        self.exec.exec(context, &mut children)
    }
    fn child_append(&mut self, child: AChildP) -> bool {
        if match self.exec.children_max() {
            ChildCount::None => false,
            ChildCount::Some(v) => self.children.count() < v,
            ChildCount::Inf => true,
        } {
            self.children.push_back(child);
            true
        } else {
            false
        }
    }
}

impl GraphNodeWrapper {
    pub fn new(exec: UniqPtr<dyn GraphExec>) -> Self {
        Self {
            exec,
            children: LList::new(),
        }
    }
}

impl NChildGraphNodeWrapper {
    pub fn new(exec: UniqPtr<dyn GraphExec>) -> Self {
        Self {
            exec,
            children: LList::new(),
            index_children: LList::new(),
        }
    }

    pub fn index_child_append(&mut self, child: AIndexChildP) {
        self.index_children.push_back(child);
    }
}

impl GraphNode for NChildGraphNodeWrapper {
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool {
        let mut children = NChildren::new(&mut self.children, &mut self.index_children);
        self.exec.exec(context, &mut children)
    }
    fn child_append(&mut self, child: AChildP) -> bool {
        //only allow 1 child max
        if match self.exec.children_max() {
            ChildCount::None => false,
            ChildCount::Some(_) | ChildCount::Inf => self.children.count() == 0,
        } {
            self.children.push_back(child);
            true
        } else {
            false
        }
    }
}
