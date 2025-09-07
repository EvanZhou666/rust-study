#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // 结构体方法，rust规定第一个方法名称必须是self
    fn area(&self) -> u32 { // &self是self &Rectangle的简写，这里我们借用所有权，并不需要获取所有权。
        self.width * self.height
    }
}

#[allow(non_snake_case)]
fn calArea(rectangle: &Rectangle) -> u32 {
    return rectangle.width * rectangle.height
}

fn main() {
    println!("{}", my_string); // 正常使用
    let rect = Rectangle {width: 3, height:4};
    dbg!(&rect); // 打印rect的第一种方法
    println!("矩形:{:?}", rect); // 通常使用此方法打印结构体
    println!("矩形的面积是:{}", calArea(&rect));
    println!("使用结构体方法计算面积:{}", rect.area());
}