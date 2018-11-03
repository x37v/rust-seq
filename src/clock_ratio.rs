use binding::BindingGetP;
use context::{ChildContext, SchedContext};
use graph::{ChildCount, ChildExec, GraphExec};

pub struct ClockRatio {
    mul: BindingGetP<u8>,
    div: BindingGetP<u8>,
}

impl ClockRatio {
    pub fn new_p(mul: BindingGetP<u8>, div: BindingGetP<u8>) -> Box<Self> {
        Box::new(Self { mul, div })
    }
}

impl GraphExec for ClockRatio {
    fn exec(&mut self, context: &mut dyn SchedContext, children: &mut dyn ChildExec) -> bool {
        let div = self.div.get() as usize;

        if div > 0 && context.context_tick() % div == 0 {
            let mul = self.mul.get() as usize;
            let period_micros = (context.context_tick_period_micros() * div as f32) / mul as f32;
            let offset = mul * (context.context_tick() / div);
            for i in 0..mul {
                let tick = offset + i;
                //XXX child context 'base tick' should be offset by (i * period_micros)
                let mut ccontext = ChildContext::new(context, tick, period_micros);
                children.exec_all(&mut ccontext);
            }
        }
        //remove self if we have no children
        children.has_children()
    }

    fn children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}