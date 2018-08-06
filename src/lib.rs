use std::ptr;
use std::mem;

#[repr(u8)]
#[derive(Copy, Clone)]
enum Color {
    R,
    G,
    B
}

pub struct GC {
    allocd: *mut ObjHeader,
    tail: *mut ObjHeader,
    alloc_color: Color,
}

impl GC {
    pub fn new() -> GC {
        GC {
            allocd: ptr::null_mut(),
            tail: ptr::null_mut(),
            alloc_color: Color::R,
        }
    }

    fn alloc_obj<T>(&self, data: T) -> (*mut Obj<T>, *mut ObjHeader) {
        let obj_ptr = Box::into_raw(Box::new(Obj {

            header: ObjHeader {
                color: self.alloc_color,
                next: ptr::null_mut(),
                block_size: mem::size_of::<T>(),
            },

            data: data,
        }));

        let header_ptr: *mut ObjHeader = unsafe {
                &mut ((*obj_ptr).header) as *mut ObjHeader
        };

        (obj_ptr, header_ptr)
    }

    pub fn alloc<T>(&mut self, data: T) -> GCObj<T> {

        if self.allocd.is_null() {
            // No allocd objects

            let (obj_ptr, header_ptr) = self.alloc_obj(data);

            self.allocd = header_ptr;
            self.tail = header_ptr;

            GCObj {
                obj: obj_ptr
            }
        } else {
            // Some allocd objects
            
            unsafe {
                // Alloc new object and insert at the end of the list
                let (obj_ptr, header_ptr) = self.alloc_obj(data);
                
                (*self.tail).next = header_ptr;
                self.tail = header_ptr;

                GCObj {
                    obj: obj_ptr,
                }
            }
        }
    }
}

pub struct GCObj<T> {
    obj: *mut Obj<T>,
}

#[repr(C)]
struct ObjHeader {
    color: Color,
    next: *mut ObjHeader,
    block_size: usize,
}

#[repr(C)]
struct Obj<T> {
    header: ObjHeader,
    data: T,
}
