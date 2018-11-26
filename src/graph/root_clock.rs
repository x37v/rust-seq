use super::*;

pub type Micro = f32;
pub struct RootClock {
    tick: usize,
    tick_sub: f32,
    period_micros: BindingGetP<Micro>,
    children: ChildList,
}

impl RootClock {
    pub fn new(period_micros: BindingGetP<Micro>) -> Self {
        Self {
            tick: 0,
            tick_sub: 0f32,
            period_micros,
            children: LList::new(),
        }
    }
    pub fn child_append(&mut self, child: AChildP) {
        self.children.push_back(child);
    }
}

impl SchedCall for RootClock {
    fn sched_call(&mut self, context: &mut dyn SchedContext) -> TimeResched {
        let period_micros = self.period_micros.get();
        if self.children.count() > 0 {
            let mut ccontext = ChildContext::new(context, 0, self.tick, period_micros);
            let mut tmp = LList::new();
            std::mem::swap(&mut self.children, &mut tmp);

            for c in tmp.into_iter() {
                if c.lock().exec(&mut ccontext) {
                    self.children.push_back(c);
                }
            }
        }

        let ctp = context.context_tick_period_micros();
        if period_micros <= 0f32 || ctp <= 0f32 {
            TimeResched::ContextRelative(1)
        } else {
            let next = self.tick_sub + (period_micros / ctp);
            self.tick_sub = next.fract();
            self.tick += 1;

            //XXX what if next is less than 1?
            assert!(next >= 1f32, "tick less than sample size not supported");
            TimeResched::ContextRelative(std::cmp::max(1, next.floor() as usize))
        }
    }
}
