#![feature(nll)]

extern crate spinlock;
extern crate xnor_llist;

use std::sync::Arc;
use xnor_llist::List;
use xnor_llist::Node as LNode;

pub type ANodeP<T> = Arc<spinlock::Mutex<Node<T>>>;
pub type AChildP<T> = Box<LNode<ANodeP<T>>>;

pub struct Node<T> {
    data: T,
    children: List<ANodeP<T>>,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Node {
            data,
            children: List::new(),
        }
    }

    pub fn new_p(data: T) -> ANodeP<T> {
        Arc::new(spinlock::Mutex::new(Self::new(data)))
    }

    pub fn new_child(node: ANodeP<T>) -> AChildP<T> {
        LNode::new_boxed(node)
    }

    pub fn children(&self) -> &List<ANodeP<T>> {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut List<ANodeP<T>> {
        &mut self.children
    }

    pub fn traverse<F>(&self, f: &F)
    where
        F: Fn(&T),
    {
        f(&self.data);
        for c in self.children.iter() {
            c.lock().traverse(f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn it_works() {
        let n = Node::new_p(1);
        let v = Node::new_p(2);
        let x = v.clone();
        {
            let z = Node::new_p(20);
            v.lock().children_mut().push_front(Node::new_child(z));

            let mut g = n.lock();
            g.children_mut().push_front(Node::new_child(x));

            g.children_mut().push_front(Node::new_child(v.clone()));
            g.children_mut().push_front(Node::new_child(v));
        }
        n.lock().traverse(&|d| println!("node: {}", d));
    }

    #[test]
    fn threaded() {
        let n = Node::new_p(1);
        let x = n.clone();
        let c = Node::new_child(Node::new_p(4345));
        let child = thread::spawn(move || {
            let mut g = n.lock();
            g.children_mut().push_back(Node::new_child(Node::new_p(20)));
            g.children_mut().push_front(c);
        });
        x.lock()
            .children_mut()
            .push_back(Node::new_child(Node::new_p(200)));
        assert!(child.join().is_ok());
        x.lock().traverse(&|d| println!("node: {}", d));
    }
}
