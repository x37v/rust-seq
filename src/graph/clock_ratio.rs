use core::ops::Deref;
use core::marker::PhantomData;
use crate::graph::{GraphNodeExec, GraphChildExec, ChildCount};
use crate::event::EventEvalContext;
use num::cast::NumCast;

pub struct ClockRatio<T, Mul, Div> {
    mul: Mul,
    div: Div,
    phantom: PhantomData<T>
}

impl<T, Mul, Div> ClockRatio<T, Mul, Div>
where
    Mul: Deref<Target=T> + Send,
    Div: Deref<Target=T> + Send,
    T: num::Unsigned + NumCast + Send
{
    pub fn new(mul: Mul, div: Div) -> Self {
        Self { mul, div, phantom: PhantomData }
    }
}

impl<T, Mul, Div> GraphNodeExec for ClockRatio<T, Mul, Div>
where
    Mul: Deref<Target=T> + Send,
    Div: Deref<Target=T> + Send,
    T: num::Unsigned + NumCast + Send
{
    fn graph_exec(&mut self, context: &mut dyn EventEvalContext, children: &mut dyn GraphChildExec)  {
        let div: usize = NumCast::from(*self.div).expect("T should cast to usize");

        if div > 0 && context.context_tick_now() % div == 0 {
            let mul: usize = NumCast::from(*self.mul).expect("T should cast to usize");
            let base_period_micros = context.tick_period_micros();
            let period_micros = (context.context_tick_period_micros() * div as f32) / mul as f32;
            let offset = (mul * context.context_tick_now()) / div;
            for i in 0..mul {
                let tick = offset + i;
                let base = ((i as f32 * period_micros) / base_period_micros) as isize;
                let mut ccontext = ChildContext::new(context, base, tick, period_micros);
                children.child_exec_all(&mut ccontext);
            }
        }
    }

    fn graph_children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}
