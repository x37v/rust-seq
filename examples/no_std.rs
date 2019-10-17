#![no_std]
#![no_main]

use sched::tick::*;

/*
use sched::context::SchedContext;
use sched::graph::GraphExec;
use sched::graph::{ChildCount, ChildExec};
use sched::tick::TickSched;

struct S;

impl GraphExec for S {
    fn exec(&mut self, _context: &mut dyn SchedContext, _children: &mut dyn ChildExec) -> bool {
        false
    }
    fn children_max(&self) -> ChildCount {
        ChildCount::Inf
    }
}
*/

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let t = TickSched::Absolute(234);

    // Since we are passing a C string the final null character is mandatory
    const HELLO: &'static str = "tick %d!\n\0";
    let (_, v) = match t {
        TickSched::Absolute(v) => ("absolute", v as isize),
        TickSched::Relative(v) => ("relative", v),
        TickSched::ContextAbsolute(v) => ("context_absolute", v as isize),
        TickSched::ContextRelative(v) => ("context_relative", v),
    };
    unsafe {
        libc::printf(HELLO.as_ptr() as *const _, v);
    }
    0
}
