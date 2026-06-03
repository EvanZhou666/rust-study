use std::sync::mpsc;
use std::thread;

fn main() {
    println!("并发channel测试!");
    let (tx, rx) = mpsc::channel::<String>();

    /*单生产者*/
    /*    let sender = thread::spawn(move || { // move将所有权转移到另外一个线程
        for i in 0..4 {
            tx.send(String::from(format!("message {}", i))).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
        drop(tx); // 关闭发送端

    });*/

    let mut senders = vec![];
    /*多生产者-单消费者*/
    /* rust标准库,并不支持多生产者和多消费者*/
    for i in 0..4 {
        let tx2 = tx.clone(); // 拷贝句柄,返回新的对象,以规避所有权检查.
        let sender = thread::spawn(move || {
            // move将所有权转移到另外一个线程
            tx2.send(String::from(format!("message {}", i))).unwrap();
            // thread::sleep(Duration::from_millis(100));
        });
        senders.push(sender);
    }

    let receiver = thread::spawn(move || {
        for msg in rx {
            // channel 关闭后，循环会自动结束，和go一样
            println!("----接收到数据:{}", msg)
        }
        // let msg = rx.recv().unwrap();
    });

    for h in senders {
        h.join().unwrap();
    }
    drop(tx); // 关闭所有的发送者,从而关闭通道
    receiver.join().unwrap();
}
