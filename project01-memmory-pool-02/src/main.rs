use std::mem::{align_of, size_of};

pub struct Arena {
    buffer: Vec<u8>,
    offset: usize,
}

impl Arena {
    // Self 是rust的保留关键字，代表当前类名
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            offset: 0,
        }
    }

    pub fn alloc<T>(&mut self, value: T) -> &mut T {
        println!("offset ={}", self.offset);
        let size = size_of::<T>();
        let align = align_of::<T>();

        let start = (self.offset + align - 1) & !(align - 1);
        let end = start + size;

        if end > self.buffer.capacity() {
            panic!("Arena out of memory");
        }

        // 扩展 vec 长度（不重新分配）
        if end > self.buffer.len() {
            self.buffer.resize(end, 0);
        }

        let ptr = unsafe {
            self.buffer.as_mut_ptr().add(start) as *mut T
        };

        unsafe {
            ptr.write(value);
        }

        self.offset = end;

        unsafe { &mut *ptr }
    }
}

fn main() {
    let mut arena = Arena::new(1024);

    {
        let a = arena.alloc(10);
        *a += 1;
        println!("{:?}", a);
    }

    {
        let b = arena.alloc(20i64);
        *b += 2;
        println!("{}", b);
    }

    {
        let c = arena.alloc(20i64);
        *c += 2;
        println!("{}", c);
    }

}