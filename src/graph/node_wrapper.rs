use super::*;
use crate::ptr::UniqPtr;

pub struct GraphNodeWrapper<C>
where
    C: ChildListT,
{
    exec: UniqPtr<dyn GraphExec>,
    children: C,
}

pub struct NChildGraphNodeWrapper<C, I>
where
    C: ChildListT,
    I: IndexChildListT,
{
    exec: UniqPtr<dyn GraphExec>,
    children: C,
    index_children: I,
}

impl<C> GraphNode for GraphNodeWrapper<C>
where
    C: ChildListT,
{
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool {
        let mut children = Children::new(&mut self.children);
        self.exec.exec(context, &mut children)
    }
    fn child_append(&mut self, child: ANodeP) -> bool {
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

impl<C> GraphNodeWrapper<C>
where
    C: ChildListT,
{
    pub fn new(exec: UniqPtr<dyn GraphExec>, children: C) -> Self {
        Self { exec, children }
    }
}

impl<C, I> NChildGraphNodeWrapper<C, I>
where
    C: ChildListT,
    I: IndexChildListT,
{
    pub fn new(exec: UniqPtr<dyn GraphExec>, children: C, index_children: I) -> Self {
        Self {
            exec,
            children,
            index_children,
        }
    }

    #[cfg(feature = "std")]
    pub fn index_child_append(&mut self, child: AIndexNodeP) {
        self.index_children.push_back(child);
    }
}

impl<C, I> GraphNode for NChildGraphNodeWrapper<C, I>
where
    C: ChildListT,
    I: IndexChildListT,
{
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool {
        let mut children = NChildren::new(&mut self.children, &mut self.index_children);
        self.exec.exec(context, &mut children)
    }

    fn child_append(&mut self, child: ANodeP) -> bool {
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
