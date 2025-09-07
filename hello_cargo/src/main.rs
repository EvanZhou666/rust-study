use std::io;

use rand::{rng, Rng};

fn main() {
    println!("-----------这是一个数字猜谜游戏-----------");
    let number: u32 = rng().random_range(1..=100);
    let mut running: bool = true;
    while running {
        let mut guess = String::new();
        println!("请输入你猜的的数字：{guess}");
        io::stdin().read_line(&mut guess).expect("读取输入失败:");
        // 变量遮蔽，重用guess变量名，但是类型变化了
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };
        match guess.cmp(&number) {
            std::cmp::Ordering::Equal => {
                running = false;
                println!("恭喜你，猜对了！");
            },
            std::cmp::Ordering::Greater => println!("很遗憾，您猜大了"),
            std::cmp::Ordering::Less => println!("很遗憾，您猜小了"),
        }
    }

    println!("The secret number is: {number}");
}
