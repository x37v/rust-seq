use crate::{
    binding::*,
    context::ChildContext,
    event::EventEvalContext,
    graph::{ChildCount, GraphChildExec, GraphNodeExec},
    Float,
};

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
            let period_micros =
                (context.context_tick_period_micros() * div as Float) / mul as Float;
            let coffset = (mul * context.context_tick_now()) / div;
            let mut ccontext = ChildContext::new(context, 0, coffset, period_micros);
            for i in 0..mul {
                ccontext.update_parent_offset(
                    ((i as Float * period_micros) / base_period_micros) as isize,
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

#[cfg(test)]
pub mod tests {
    extern crate alloc;
    use super::*;

    use crate::graph::{node_wrapper::GraphNodeWrapper, *};
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use core::sync::atomic::AtomicUsize;

    use crate::context::tests::TestContext;

    #[test]
    fn radio() {
        let mut context = TestContext::new(0, 44100);
        let tick = Arc::new(AtomicUsize::new(0));
        let ctick = Arc::new(AtomicUsize::new(0));
        let mul = Arc::new(AtomicUsize::new(1));
        let div = Arc::new(AtomicUsize::new(1));

        let store_ctick: GraphNodeContainer = GraphNodeWrapper::new(
            tick_record::TickRecord::Context(ctick.clone() as Arc<dyn ParamBindingSet<usize>>),
            children::empty::Children,
        )
        .into();

        let store_tick: GraphNodeContainer = GraphNodeWrapper::new(
            tick_record::TickRecord::Absolute(tick.clone() as Arc<dyn ParamBindingSet<usize>>),
            children::empty::Children,
        )
        .into();

        let mut ratio: GraphNodeContainer = GraphNodeWrapper::new(
            ClockRatio::new(
                mul.clone() as Arc<dyn ParamBindingGet<_>>,
                div.clone() as Arc<dyn ParamBindingGet<_>>,
            ),
            children::boxed::Children::new(Box::new([store_tick, store_ctick])),
        )
        .into();

        //trigger the clock ratio and verify the tick output
        assert_eq!(0, tick.get());
        assert_eq!(0, ctick.get());

        ratio.node_exec(&mut context);
        assert_eq!(0, tick.get());
        assert_eq!(0, ctick.get());

        context.set_tick(1);
        ratio.node_exec(&mut context);
        assert_eq!(1, tick.get());
        assert_eq!(1, ctick.get());

        ratio.node_exec(&mut context);
        assert_eq!(1, tick.get());
        assert_eq!(1, ctick.get());
        for i in 0..20 {
            context.set_tick(i);
            ratio.node_exec(&mut context);
            assert_eq!(i, tick.get());
            assert_eq!(i, ctick.get());
        }

        //should only trigger every other time
        div.set(2);
        context.set_tick(0);
        ratio.node_exec(&mut context);
        assert_eq!(0, tick.get());
        assert_eq!(0, ctick.get());

        context.set_tick(1);
        ratio.node_exec(&mut context);
        assert_eq!(0, tick.get());
        assert_eq!(0, ctick.get());

        context.set_tick(2);
        ratio.node_exec(&mut context);
        assert_eq!(2, tick.get());
        assert_eq!(1, ctick.get());

        context.set_tick(3);
        ratio.node_exec(&mut context);
        assert_eq!(2, tick.get());
        assert_eq!(1, ctick.get());

        context.set_tick(4);
        ratio.node_exec(&mut context);
        assert_eq!(4, tick.get());
        assert_eq!(2, ctick.get());

        div.set(4);
        ratio.node_exec(&mut context);
        assert_eq!(4, tick.get());
        assert_eq!(1, ctick.get());

        context.set_tick(5);
        ratio.node_exec(&mut context);
        assert_eq!(4, tick.get());
        assert_eq!(1, ctick.get());

        context.set_tick(8);
        ratio.node_exec(&mut context);
        assert_eq!(8, tick.get());
        assert_eq!(2, ctick.get());
    }
}
