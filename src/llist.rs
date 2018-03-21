use std::sync::Arc;

pub struct LLNode<T> {
    next: Option<Arc<LLNode<T>>>,
    value: T
}

pub struct LList<T> {
    head: Option<Arc<LLNode<T>>>
}

impl<T> LLNode<T> {
    fn new(v: T) -> Self {
        LLNode { next: None, value: v }
    }

    fn append(&mut self, item: Arc<LLNode<T>>) {
        self.next = Some(item);
    }
}

impl<T> LList<T> {
    pub fn new() -> Self {
        LList { head: None }
    }

    pub fn append(&mut self, node: Arc<LLNode<T>>) -> () {
        if let Some(ref mut n) = self.head {
            if let Some(r) = Arc::get_mut(n) {
                r.append(node);
            }
        } else {
            self.head = Some(node);
        }
    }
}

