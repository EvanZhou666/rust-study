use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

// 无锁并发计数器
// 在rust中，atomic提供了硬件级别的原子性，不需要锁
fn main() {
    let counterArc = Arc::new(AtomicUsize::new(0));
    let mut handlers = vec![];
    for _ in 0..10 {
        let arc = Arc::clone(&counterArc); // 使用arc增加引用计数
        handlers.push(thread::spawn(move || {
            arc.fetch_add(1, Ordering::SeqCst);
        }));
    }

    for h in handlers {
        h.join().unwrap();
    }

    println!("{:?}", counterArc)

}
