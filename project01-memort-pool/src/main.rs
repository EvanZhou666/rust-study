use std::alloc::{alloc, dealloc, Layout};
use std::cell::UnsafeCell;
use std::ptr::NonNull;

pub struct Arena {
    ptr: NonNull<u8>,
    capacity: usize,
    offset: UnsafeCell<usize>,
}

// 关键：Arena 内部有 UnsafeCell，所以我们必须自己保证线程安全
unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 8).unwrap();
        let ptr = unsafe { alloc(layout) };
        let ptr = NonNull::new(ptr).expect("allocation failed");

        Arena {
            ptr,
            capacity,
            offset: UnsafeCell::new(0),
        }
    }

    /// # Safety
    ///
    /// - 调用者必须保证：
    ///   - 不同时通过返回的 &mut T 产生别名
    ///   - Arena 在所有引用存活期间不被 drop
    pub unsafe fn alloc<T>(&self, value: T) -> &mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        let offset = &mut *self.offset.get();

        let start = (*offset + align - 1) & !(align - 1);

        if start + size > self.capacity {
            panic!("Arena out of memory");
        }

        let ptr = self.ptr.as_ptr().add(start) as *mut T;
        ptr.write(value);

        *offset = start + size;

        &mut *ptr
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 8).unwrap();
        unsafe {
            dealloc(self.ptr.as_ptr(), layout);
        }
    }
}

fn main() {
    let arena = Arena::new(1024);

    unsafe {
        let a = arena.alloc(10);
        let b = arena.alloc(20);

        *a += 1;
        *b += 2;

        println!("{}, {}", a, b); // 11, 22
    }
}
