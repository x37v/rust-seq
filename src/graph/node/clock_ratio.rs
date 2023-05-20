use crate::{
    context::ChildContext,
    event::EventEvalContext,
    graph::{GraphChildExec, GraphNodeExec},
    param::ParamGet,
    Float,
};

use num_traits::cast::NumCast;

pub struct ClockRatio<T, Mul, Div, U> {
    mul: Mul,
    div: Div,
    _phantom: core::marker::PhantomData<(T, U)>,
}

impl<T, Mul, Div, U> ClockRatio<T, Mul, Div, U>
where
    Mul: ParamGet<T, U>,
    Div: ParamGet<T, U>,
    T: num_traits::sign::Unsigned + NumCast,
{
    pub fn new(mul: Mul, div: Div) -> Self {
        Self {
            mul,
            div,
            _phantom: Default::default(),
        }
    }
}

impl<T, Mul, Div, E, U> GraphNodeExec<E, U> for ClockRatio<T, Mul, Div, U>
where
    Mul: ParamGet<T, U>,
    Div: ParamGet<T, U>,
    T: num_traits::sign::Unsigned + NumCast,
{
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    ) {
        let div: usize = NumCast::from(self.div.get(user_data)).expect("T should cast to usize");

        if div > 0 && context.context_tick_now() % div == 0 {
            let mul: usize =
                NumCast::from(self.mul.get(user_data)).expect("T should cast to usize");
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
                children.child_exec_all(&mut ccontext, user_data);
            }
        }
    }
}

/*
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

        let ratio: GraphNodeContainer = GraphNodeWrapper::new(
            ClockRatio::new(
                mul.clone() as Arc<dyn ParamGet<_>>,
                div.clone() as Arc<dyn ParamGet<_>>,
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
*/
