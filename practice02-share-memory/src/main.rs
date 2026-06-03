use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // let mutex = Mutex::new(0);
    // let mutex_guard = mutex.lock().unwrap(); // mutexGuard实现了Deref解引用trait，会指向内部真实的数据
    // println!("{}", mutex_guard);

    // let mut a = 128;
    // a = a + 1;
    // println!("a: {}", a);

    let mut handlers = vec![];
    // Arc原子引用计数,适合多线程
    let arc = Arc::new(Mutex::new(0));
    for _i in 0..4 {
        let arc = Arc::clone(&arc); // 增加引用计数,不是拷贝
        let h = thread::spawn(move || {
            let mut num = arc.lock().unwrap();
            *num = *num + 1;
        });
        handlers.push(h);
    }

    for h in handlers {
        h.join().unwrap();
    }
    println!("{}", arc.lock().unwrap());

}
