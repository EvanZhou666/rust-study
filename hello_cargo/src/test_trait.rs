// 测试triat

// 定义trait接口
pub trait Shape {
    fn draw(&self);
}

struct Rectangle{
    x: i32,
    y: i32,
}
// 为矩形实现Shape 接口
impl Shape for Rectangle {
    fn draw(&self) {
        println!("绘制矩形，长={}，宽={}", self.x, self.y);
    }
}

impl Shape for String {
     fn draw(&self) {
        println!("rust 接口更灵活，可不修改第三方代码实现接口");
    }
}

fn main() {

    let shape = Rectangle{x: 3, y: 4};
    shape.draw();

    let str = String::from("神器吧");
    str.draw();
}