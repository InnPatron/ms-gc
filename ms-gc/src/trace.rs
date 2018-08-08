#[macro_export]
macro_rules! empty_trace {
    ($($t: ty,)*) => {
        $(
            unsafe impl Trace for $t { fn trace(&self) { } }
        )*
    }
}

pub unsafe trait Trace {
    fn trace(&self);
}

empty_trace!(i8, i16, i32, i64, 
             u8, u16, u32, u64, 
             usize, isize, 
             bool,
             f32, f64,);

unsafe impl<T: Trace> Trace for Vec<T> {
    fn trace(&self) {
        for e in self {
            e.trace();
        }
    }
}

unsafe impl<T: Trace> Trace for Option<T> {
    fn trace(&self) {
        match self {
            &Some(ref traceable) => traceable.trace(),
            None => (),
        }
    }
}
