#![feature(specialization)]
//could have a key -> value where the value is an enumerated item of any of the types we support?
//not super flexible..

type UTime = usize;
type SeqFn = Box<Fn(&mut Seq) -> Option<UTime>>;

trait SeqSend {
  fn send_usize(&mut self, v: usize) -> ();
}

impl<T> SeqSend for T {
  default fn send_usize(&mut self, _v: usize) { ; }
}

impl SeqSend for Seq {
  fn send_usize(&mut self, v: usize) -> () { 
    println!("YES {}", v);
  }
}

/*
struct LLNode<T> {
    next: Option<Box<LLNode<T>>>,
    value: T
}

impl<T> LLNode<T> {
    fn new(v: T) -> Self {
        LLNode { next: None, value: v }
    }

    fn append(&mut self, item: Box<LLNode<T>>) {
        self.next = Some(item);
    }
}
*/

// Fn(context) -> option(utime) [if it gets rescheduled or not]
// context allows for scheduling additional things

struct Seq { 
    items: Vec<SeqFn>,
    reserve: Vec<SeqFn>
}

impl Seq {
    fn new() -> Self {
        Seq {
            items: Vec::new(),
            reserve: Vec::new()
        }
    }

    fn schedule(&mut self, f: SeqFn) {
        self.items.push(f);
    }

    fn reserve(&mut self, f: SeqFn) {
        self.reserve.push(f);
    }

    fn reserve_pop(&mut self) -> Option<SeqFn> {
        self.reserve.pop()
    }

    fn run(&mut self) {
        //XXX loop while we still have items in the current time slice.
        //abort early if it takes too long?
        println!("run!");
        let l: Vec<SeqFn> = self.items.drain(..).collect();
        for f in l {
            if let Some(n) = f(self) {
                println!("{}", n);
                self.items.push(f);
            }
        }
    }
}

fn main() {
    let mut seq = Seq::new();

    for i in 1..10 {
        seq.reserve(Box::new(move |_s: &mut Seq| {
            Some(i)
        }));
    }

    seq.send_usize(30);

    seq.schedule(Box::new(|s: &mut Seq| {
        let v = Box::new(|s: &mut Seq| {
            if let Some(n) = s.reserve_pop() {
                s.schedule(n);
            }
            Some(20)
        });
        s.schedule(v);
        None
    }));

    for _ in 1..10 {
        seq.run();
    }
}
