extern crate alloc;

use crate::binding::*;
use crate::event::{EventContainer, EventEvalContext};
use crate::graph::root_event::RootEvent;
use crate::graph::GraphNode;
use crate::item_source::ItemSource;
use crate::tick::{TickResched, TickSched};
use alloc::sync::Arc;
use spin::Mutex;

pub struct Notify<B>
where
    B: 'static + ParamBindingGet<TickResched>,
{
    gate_open: bool,
    next_binding: B,
}

impl<B> Notify<B>
where
    B: 'static + ParamBindingGet<TickResched>,
{
    pub fn new(next_binding: B) -> Self {
        Self {
            gate_open: true,
            next_binding,
        }
    }

    pub fn test_and_set(&mut self) -> bool {
        if self.gate_open {
            self.gate_open = false;
            true
        } else {
            false
        }
    }
}

impl<B> ParamBindingGet<TickResched> for Arc<Mutex<Notify<B>>>
where
    B: 'static + ParamBindingGet<TickResched>,
{
    fn get(&self) -> TickResched {
        let mut l = self.lock();
        let v = l.next_binding.get();
        if v == TickResched::None {
            l.gate_open = true;
        }
        v
    }
}

pub struct RetrigScheduler<T, B, S>
where
    T: 'static + GraphNode + Clone,
    B: 'static + ParamBindingGet<TickResched> + Clone,
    S: 'static + ItemSource<RootEvent<T, Arc<Mutex<Notify<B>>>>>,
{
    node: T,
    next_binding: Arc<Mutex<Notify<B>>>,
    source: S,
}

impl<T, B, S> RetrigScheduler<T, B, S>
where
    T: 'static + GraphNode + Clone,
    B: 'static + ParamBindingGet<TickResched> + Clone,
    S: 'static + ItemSource<RootEvent<T, Arc<Mutex<Notify<B>>>>>,
{
    pub fn new(node: T, next_binding: B, source: S) -> Self {
        Self {
            node,
            next_binding: Arc::new(Mutex::new(Notify::new(next_binding))),
            source,
        }
    }
}

impl<T, B, S> GraphNode for RetrigScheduler<T, B, S>
where
    T: 'static + GraphNode + Clone,
    B: 'static + ParamBindingGet<TickResched> + Clone,
    S: 'static + ItemSource<RootEvent<T, Arc<Mutex<Notify<B>>>>>,
{
    fn node_exec(&self, context: &mut dyn EventEvalContext) {
        //if the gate is open, close it and schedule the node
        if self.next_binding.lock().test_and_set() {
            if let Ok(e) = self
                .source
                .try_get(RootEvent::new(self.node.clone(), self.next_binding.clone()))
            {
                if context
                    .event_schedule(
                        TickSched::ContextRelative(0),
                        EventContainer::new_from_box(e),
                    )
                    .is_err()
                {
                    //XXX report error
                }
            } else {
                //XXX report error
            }
        }
    }
}
