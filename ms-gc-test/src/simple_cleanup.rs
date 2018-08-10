use ms_gc::*;

#[test]
fn simple_cleanup() {
    let mut gc = GC::new();
    let o1 = gc.alloc(5);
    let o2 = gc.alloc(10);
    let o3 = gc.alloc(15);

    unsafe { gc.sweep(); }
}
