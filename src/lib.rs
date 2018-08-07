use std::ptr;
use std::mem;
use std::alloc::{alloc, dealloc, Layout};

pub struct GC {
    allocd: *mut ObjHeader,
    tail: *mut ObjHeader,
}

impl GC {
    pub fn new() -> GC {
        GC {
            allocd: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    unsafe fn alloc_obj<T>(&self, data: *const T) -> (*mut Obj<T>, *mut ObjHeader) {
        let fake_ptr = data as *const Obj<T>;

        // Layout for Obj<T>
        let layout = Layout::for_value(&*fake_ptr);

        // Allocate Obj<T>
        let obj_ptr = { 
            let ptr = alloc(layout) as *mut Obj<T>;
            (*ptr).header.reachable = true;
            (*ptr).header.next = ptr::null_mut();
            (*ptr).header.block_size = mem::size_of::<T>();
            ptr
        };

        let header_ptr: *mut ObjHeader = &mut ((*obj_ptr).header) as *mut ObjHeader;

        (obj_ptr, header_ptr)
    }

    pub fn alloc<T>(&mut self, data: T) -> GCObj<T> {
        let (obj_ptr, header_ptr) = unsafe { self.alloc_obj(&data as *const T) };
        unsafe { 
            (*obj_ptr).data = data;
        }
        if self.allocd.is_null() {
            // No allocd objects

            self.allocd = header_ptr;
            self.tail = header_ptr;

            GCObj {
                obj: obj_ptr
            }
        } else {
            // Some allocd objects
            
            // Alloc new object and insert at the end of the list
            unsafe {    
                (*self.tail).next = header_ptr;
            }
            self.tail = header_ptr;

            GCObj {
                obj: obj_ptr,
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct GCObj<T> {
    obj: *mut Obj<T>,
}

#[repr(C)]
struct ObjHeader {
    reachable: bool,
    next: *mut ObjHeader,
    block_size: usize,
}

#[repr(C)]
struct Obj<T> {
    header: ObjHeader,
    data: T,
}
