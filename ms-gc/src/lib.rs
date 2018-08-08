use std::ptr;
use std::mem;
use std::alloc::{alloc, dealloc, Layout};
use std::cell::Cell;
use std::ops::Deref;

mod trace;

pub use trace::*;


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

        } else {
            // Some allocd objects
            
            // Alloc new object and insert at the end of the list
            unsafe {    
                (*self.tail.unwrap().as_ptr()).header.next = Some(ptr::NonNull::new_unchecked(obj_ptr));
                self.tail = Some(ptr::NonNull::new_unchecked(obj_ptr));
            }
            
        }

        unsafe {
            GCObj {
                obj: ptr::NonNull::new_unchecked(obj_ptr),
            }
        }
    }

    pub fn mark(roots: &[&Trace]) {
        for root in roots {
            root.trace();
        }
    }

    pub fn sweep(&mut self) {
        // Predeccesor of the current node in the list
        let mut pred: Option<ptr::NonNull<Obj<Trace>>> = None;
        let mut current = self.allocd;

        unsafe {
            while let Some(obj_ptr) = current {

                let obj_ptr = obj_ptr.as_ptr();
                if (*obj_ptr).header.reachable.get() {
                    // Object is still live
                    // Reset reachable flag and continue through the list
                    (*obj_ptr).header.reachable.set(false);
                    current = (*obj_ptr).header.next;

                    // Update
                    pred = current;
                } else {

                    // Object is NOT live
                    // Remove object from the list
                    match pred {
                        Some(pred) => {
                            (*pred.as_ptr()).header.next = (*obj_ptr).header.next;
                        }

                        None => (),
                    }

                    // TODO: Call the object's data destructor

                    // Deallocate the object
                    let layout = (*obj_ptr).header.layout;
                    let data_ptr = obj_ptr as *mut u8;
                    dealloc(data_ptr, layout);
                }
            }
        }
    }
}

pub struct GCObj<T: Trace + ?Sized + 'static> {
    obj: ptr::NonNull<Obj<T>>
}

impl<T: Trace + ?Sized + 'static> Clone for GCObj<T> {
    fn clone(&self) -> GCObj<T> {
        GCObj {
            obj: self.obj
        }
    }
}

#[repr(C)]
struct ObjHeader {
    reachable: Cell<bool>,
    next: Option<ptr::NonNull<Obj<Trace>>>,
    layout: Layout,
}

impl<T: Trace + ?Sized + 'static> Deref for GCObj<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &(*self.obj.as_ptr()).data
        }
    }
}

#[repr(C)]
struct Obj<T: Trace + ?Sized + 'static> {
    header: ObjHeader,
    data: T,
}

unsafe impl<T: Trace + ?Sized + 'static> Trace for GCObj<T> {
    fn trace(&self) {
        unsafe {
            if !(*self.obj.as_ptr()).header.reachable.get() {
                // Object and children have not been traced yet
                (*self.obj.as_ptr()).header.reachable.set(true);
                (*self.obj.as_ptr()).data.trace();
            }
            // Otherwise assume object and children have been traced
        }
    }
}
