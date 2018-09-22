extern crate spinlock;
extern crate xnor_llist;

use base::{Context, LList, SchedContext, SrcSink};
use std::sync::Arc;
use xnor_llist::List;
use xnor_llist::Node as LNode;

pub trait GraphExec: Send {
    fn exec(&self, context: &mut Context) -> bool;
    fn child_append(&mut self, child: AChildP);
}

pub type ANodeP = Arc<spinlock::Mutex<dyn GraphExec>>;
pub type AChildP = Box<LNode<ANodeP>>;
pub type ChildList = List<ANodeP>;

#[cfg(test)]
mod tests {
    use super::*;
    use std;
    use std::vec::Vec;

    struct X {
        children: ChildList,
    }
    struct Y {}

    impl GraphExec for X {
        fn exec(&self, context: &mut Context) -> bool {
            println!("XES");
            for c in self.children.iter() {
                c.lock().exec(context);
            }
            false
        }
        fn child_append(&mut self, child: AChildP) {
            self.children.push_back(child);
        }
    }

    impl GraphExec for Y {
        fn exec(&self, _context: &mut Context) -> bool {
            println!("YES");
            true
        }
        fn child_append(&mut self, child: AChildP) {}
    }

    #[test]
    fn with_mutex() {
        type M<T> = std::sync::Mutex<T>;

        let x = Arc::new(M::new(X {
            children: List::new(),
        }));
        let y = Arc::new(M::new(Y {}));

        let mut l: LList<std::sync::Arc<M<dyn GraphExec>>> = List::new();
        l.push_back(LNode::new_boxed(y.clone()));
        l.push_back(LNode::new_boxed(x.clone()));

        let mut v: Vec<Box<LNode<std::sync::Arc<M<dyn GraphExec>>>>> = Vec::new();
        v.push(LNode::new_boxed(y));
        v.push(LNode::new_boxed(x));

        let mut src_sink = SrcSink::new();
        let mut list = LList::new();

        let mut c = Context::new_root(0, 0, &mut list, &mut src_sink);

        for i in l.iter() {
            let g = i.lock().unwrap();
            g.exec(&mut c);
        }

        for i in v.iter() {
            let g = i.lock().unwrap();
            g.exec(&mut c);
        }
    }

    #[test]
    fn with_my_mutex() {
        type M<T> = spinlock::Mutex<T>;

        let x = Arc::new(M::new(X {
            children: List::new(),
        }));
        let y = Arc::new(M::new(Y {}));

        let mut l: LList<std::sync::Arc<M<dyn GraphExec>>> = List::new();
        l.push_back(LNode::new_boxed(y.clone()));
        l.push_back(LNode::new_boxed(x.clone()));

        let mut v: Vec<Box<LNode<std::sync::Arc<M<dyn GraphExec>>>>> = Vec::new();
        v.push(LNode::new_boxed(y));
        v.push(LNode::new_boxed(x));

        let mut src_sink = SrcSink::new();
        let mut list = LList::new();

        let mut c = Context::new_root(0, 0, &mut list, &mut src_sink);

        for i in l.iter() {
            let g = i.lock();
            g.exec(&mut c);
        }

        for i in v.iter() {
            let g = i.lock();
            g.exec(&mut c);
        }
    }

}
