use super::*;
use ptr::UniqPtr;

pub struct FuncWrapper<F> {
    func: UniqPtr<F>,
    children_max: ChildCount,
}

pub struct IndexFuncWrapper<F> {
    func: UniqPtr<F>,
}

impl<F> FuncWrapper<F>
where
    F: Fn(&mut dyn SchedContext, &mut dyn ChildExec) -> bool + Send,
{
    pub fn new_boxed(children_max: ChildCount, func: F) -> UniqPtr<Self> {
        new_uniqptr!(Self {
            func: new_uniqptr!(func),
            children_max,
        })
    }
}

impl<F> GraphExec for FuncWrapper<F>
where
    F: Fn(&mut dyn SchedContext, &mut dyn ChildExec) -> bool + Send,
{
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
        (self.func)(context, children)
    }

    fn children_max(&self) -> ChildCount {
        self.children_max
    }
}

impl<F> IndexFuncWrapper<F>
where
    F: Fn(usize, &mut dyn SchedContext) + Send,
{
    pub fn new(func: F) -> Self {
        Self {
            func: new_uniqptr!(func),
        }
    }
}

impl<F> GraphIndexExec for IndexFuncWrapper<F>
where
    F: Fn(usize, &mut dyn SchedContext) + Send,
{
    fn exec_index(&mut self, index: usize, context: &mut dyn SchedContext) {
        (self.func)(index, context);
    }
}
