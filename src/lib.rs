use std::ptr;
use std::mem;
use std::alloc::{alloc, dealloc, Layout};
use std::cell::Cell;

pub struct GC {
    allocd: Option<ptr::NonNull<Obj<Trace>>>,
    tail: Option<ptr::NonNull<Obj<Trace>>>,
}

impl GC {
    pub fn new() -> GC {
        GC {
            allocd: None,
            tail: None,
        }
    }

    unsafe fn alloc_obj<T: Trace + 'static>(&self, data: T) -> *mut Obj<T> {
        let fake_ptr = &data as *const T as *const Obj<T>;

        // Layout for Obj<T>
        let layout = Layout::for_value(&*fake_ptr);

        // Allocate Obj<T>
        let obj_ptr = { 
            let ptr = alloc(layout) as *mut Obj<T>;
            (*ptr).header.reachable = Cell::new(true);
            (*ptr).header.next = None;
            (*ptr).header.layout = layout;

            (*ptr).data = data;

            ptr
        };

        obj_ptr
    }

    pub fn alloc<T: Trace + 'static>(&mut self, data: T) -> GCObj<T> {
        let obj_ptr = unsafe { self.alloc_obj(data) };

        if self.allocd.is_none() {
            // No allocd objects

            unsafe {
                self.allocd = Some(ptr::NonNull::new_unchecked(obj_ptr));
                self.tail = Some(ptr::NonNull::new_unchecked(obj_ptr));
            }

            GCObj {
                obj: obj_ptr
            }
        } else {
            // Some allocd objects
            
            // Alloc new object and insert at the end of the list
            unsafe {    
                (*self.tail.unwrap().as_ptr()).header.next = Some(ptr::NonNull::new_unchecked(obj_ptr));
                self.tail = Some(ptr::NonNull::new_unchecked(obj_ptr));
            }

            GCObj {
                obj: obj_ptr,
            }
        }
    }

    pub fn mark(roots: &[&Trace]) {
        for root in roots {
            root.trace();
        }
    }
}

#[derive(Copy, Clone)]
pub struct GCObj<T: Trace + ?Sized + 'static> {
    obj: *mut Obj<T>,
}

#[repr(C)]
struct ObjHeader {
    reachable: Cell<bool>,
    next: Option<ptr::NonNull<Obj<Trace>>>,
    layout: Layout,
}

#[repr(C)]
struct Obj<T: Trace + ?Sized + 'static> {
    header: ObjHeader,
    data: T,
}

pub unsafe trait Trace {
    fn trace(&self);
}

unsafe impl<T: Trace + ?Sized + 'static> Trace for GCObj<T> {
    fn trace(&self) {
        unsafe {
            if !(*self.obj).header.reachable.get() {
                // Object and children have not been traced yet
                (*self.obj).header.reachable.set(true);
                (*self.obj).data.trace();
            }
            // Otherwise assume object and children have been traced
        }
    }
}
