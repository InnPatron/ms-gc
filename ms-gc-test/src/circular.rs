use std::cell::RefCell;

use std::ops::Drop;

use ms_gc::*;

struct Circular(RefCell<Option<GCObj<Circular>>>, i32);

unsafe impl Trace for Circular {
    fn trace(&self) {
        let b = self.0.borrow();

        match *b {
            Some(ref obj) => obj.trace(),
            None => ()
        }
    }
}

#[test]
fn circular_ref() {
    let mut gc = GC::new();
    let c1 = gc.alloc(Circular(RefCell::new(None), 0));
    let c2 = gc.alloc(Circular(RefCell::new(None), 1));

    println!("PRE: {}", (*c1).1);
    println!("PRE: {}", (*c2).1);

    {
        let mut borrow = (*c1).0.borrow_mut();
        *borrow = Some(c2.clone());
    }

    {
        let mut borrow = (*c2).0.borrow_mut();
        *borrow = Some(c1.clone());
    }

    GC::mark(vec![&c1 as &Trace, &c2 as &Trace].as_slice());
    gc.sweep();

    GC::mark(vec![&c1 as &Trace].as_slice());
    gc.sweep();

    println!("POST: {}", (*c1).1);
    println!("POST: {}", (*c2).1);

}
