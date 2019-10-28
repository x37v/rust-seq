use crate::binding::*;
use crate::context::ChildContext;
use crate::event::EventEvalContext;
use crate::graph::{ChildCount, GraphChildExec, GraphNodeExec};
use core::marker::PhantomData;
use num::cast::NumCast;

pub struct ClockRatio<T, Mul, Div> {
    mul: Mul,
    div: Div,
    phantom: PhantomData<T>,
}

impl<T, Mul, Div> ClockRatio<T, Mul, Div>
where
    Mul: ParamBindingGet<T>,
    Div: ParamBindingGet<T>,
    T: num::Unsigned + NumCast + Send,
{
    pub fn new(mul: Mul, div: Div) -> Self {
        Self {
            mul,
            div,
            phantom: PhantomData,
        }
    }
}

impl<T, Mul, Div> GraphNodeExec for ClockRatio<T, Mul, Div>
where
    Mul: ParamBindingGet<T>,
    Div: ParamBindingGet<T>,
    T: num::Unsigned + NumCast + Send,
{
    fn graph_exec(
        &mut self,
        context: &mut dyn EventEvalContext,
        children: &mut dyn GraphChildExec,
    ) {
        let div: usize = NumCast::from(self.div.get()).expect("T should cast to usize");

        if div > 0 && context.context_tick_now() % div == 0 {
            let mul: usize = NumCast::from(self.mul.get()).expect("T should cast to usize");
            let base_period_micros = context.tick_period_micros();
            let period_micros = (context.context_tick_period_micros() * div as f32) / mul as f32;
            let coffset = (mul * context.context_tick_now()) / div;
            let mut ccontext = ChildContext::new(context, 0, coffset, period_micros);
            for i in 0..mul {
                ccontext.update_parent_offset(
                    ((i as f32 * period_micros) / base_period_micros) as isize,
                );
                ccontext.update_context_tick(coffset + i);
                children.child_exec_all(&mut ccontext);
            }
        }
    }

    fn graph_children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}
