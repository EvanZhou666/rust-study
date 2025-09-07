use std::fmt::Display;
use std::cmp::PartialOrd;

// 创建泛型函数，参数的类型必须要实现了Display 和 PartialOrd triat。
fn largetest<T: Display + PartialOrd>(t1: &T, t2: &T){
    println!("这是一个泛型函数t1={},t2={}", t1, t2);
    if t1 > t2 {
        println!("两者较大者是：{}", t1);
    } else {
        println!("两者较大者是：{}", t2);
    }
}

// 测试泛型
fn main() {
     largetest(&6, &4);
     largetest(&'g', &'d');
}