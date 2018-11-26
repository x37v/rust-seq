use super::*;

pub struct FuncWrapper<F> {
    func: Box<F>,
    children_max: ChildCount,
}

pub struct IndexFuncWrapper<F> {
    func: Box<F>,
}

impl<F> FuncWrapper<F>
where
    F: Fn(&mut dyn SchedContext, &mut dyn ChildExec) -> bool + Send,
{
    pub fn new_boxed(children_max: ChildCount, func: F) -> Box<Self> {
        Box::new(Self {
            func: Box::new(func),
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
            func: Box::new(func),
        }
    }

    pub fn new_p(func: F) -> Arc<spinlock::Mutex<Self>> {
        Arc::new(spinlock::Mutex::new(Self::new(func)))
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
