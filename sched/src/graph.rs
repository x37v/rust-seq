extern crate spinlock;
extern crate xnor_llist;

use base::SchedContext;
use std::sync::Arc;
use xnor_llist::List;
use xnor_llist::Node as LNode;

pub trait GraphExec: Send {
    fn exec(&mut self, context: &mut dyn SchedContext) -> bool;
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
        fn exec(&mut self, context: &mut dyn SchedContext) -> bool {
            println!("XES");

            let mut tmp = List::new();
            std::mem::swap(&mut self.children, &mut tmp);
            for c in tmp.into_iter() {
                if c.lock().exec(context) {
                    self.children.push_back(c);
                }
            }

            self.children.count() > 0
        }
        fn child_append(&mut self, child: AChildP) {
            self.children.push_back(child);
        }
    }

    impl GraphExec for Y {
        fn exec(&mut self, _context: &mut dyn SchedContext) -> bool {
            println!("ONCE");
            false
        }
        fn child_append(&mut self, child: AChildP) {}
    }

    #[test]
    fn works() {
        type M<T> = spinlock::Mutex<T>;

        let x = Arc::new(M::new(X {
            children: List::new(),
        }));
        let y = Arc::new(M::new(Y {}));

        let mut l: LList<std::sync::Arc<M<dyn GraphExec>>> = List::new();
        l.push_back(LNode::new_boxed(x.clone()));
        x.lock().child_append(LNode::new_boxed(y.clone()));

        let mut src_sink = SrcSink::new();
        let mut list = LList::new();

        let mut c = Context::new_root(0, 0, &mut list, &mut src_sink);

        for i in l.iter() {
            i.lock().exec(&mut c);
        }

        for i in l.iter() {
            i.lock().exec(&mut c);
        }
    }

}
