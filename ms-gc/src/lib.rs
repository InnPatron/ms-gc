use std::ptr;
use std::mem;
use std::alloc::{alloc, dealloc, Layout};
use std::cell::Cell;
use std::ops::Deref;

mod trace;

pub use trace::*;


pub struct GC {
    allocd: Option<ptr::NonNull<ObjHeader>>,
    tail: Option<ptr::NonNull<ObjHeader>>,
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
            (*ptr).header.reachable = Cell::new(false);
            (*ptr).header.next = None;
            (*ptr).header.layout = layout;

            (*ptr).data = data;

            ptr
        };

        obj_ptr
    }

    pub fn alloc<T: Trace + 'static>(&mut self, data: T) -> GCObj<T> {
        let obj_ptr = unsafe { self.alloc_obj(data) };
        let header_ptr = unsafe { &(*obj_ptr).header as *const ObjHeader as *mut _ };

        if self.allocd.is_none() {
            // No allocd objects

            unsafe {
                self.allocd = Some(ptr::NonNull::new_unchecked(header_ptr));
                self.tail = Some(ptr::NonNull::new_unchecked(header_ptr));
            }

        } else {
            // Some allocd objects
            
            // Alloc new object and insert at the end of the list
            unsafe {    
                (*self.tail.unwrap().as_ptr()).next = Some(ptr::NonNull::new_unchecked(header_ptr));
                self.tail = Some(ptr::NonNull::new_unchecked(header_ptr));
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

    pub unsafe fn sweep(&mut self) {
        // Predeccesor of the current node in the list
        let mut pred = None;
        let mut current = self.allocd;

        unsafe {
            while let Some(header_ptr) = current {

                let header_ptr = header_ptr.as_ptr();
                if (*header_ptr).reachable.get() {
                    // Object is still live
                    // Reset reachable flag and continue through the list
                    (*header_ptr).reachable.set(false);

                    // Move to the next node
                    pred = current;
                    current = (*header_ptr).next;
                } else {

                    // Object is NOT live
                    // Remove object from the list
                    match pred {
                        Some(pred) => {
                            (*pred.as_ptr()).next = (*header_ptr).next;
                        }

                        None => (),
                    }

                    // Move to the next node
                    // Predecessor stays the same
                    current = (*header_ptr).next;

                    // TODO: Call the object's data destructor

                    // Deallocate the object
                    // NOTE: Deallocation scheme relies on Obj's C representation
                    let layout = (*header_ptr).layout;
                    let data_ptr = header_ptr as *mut u8;
                    dealloc(data_ptr, layout);
                }
            }
        }
    }
}

pub type GCCell<T> = GCObj<::std::cell::RefCell<T>>;

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
    next: Option<ptr::NonNull<ObjHeader>>,
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
