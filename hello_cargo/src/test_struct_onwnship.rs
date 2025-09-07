
#[derive(Debug)]
#[allow(dead_code)]
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

// 测试结构体所有权
fn main() {
    let user1 = User {
        active: true,
        username: String::from("someusername123"),
        email: String::from("someone@example.com"),
        sign_in_count: 1,
    };

    // 从其它结构体实例创建user2
    let user2 = User {
        username: String::from("someusername123"),
        ..user1  // 从user1拷贝active，sign_in_count数据，基本类型默认实现了Copy Trait，因此这里是栈上copy，不会有所有权移动问题
    };

    // println!("{:?}", user1); 编译错误: 因为email是String类型， user1内部的所有权已经移动
    // println!("Email:{}", user1.email); 编译错误: user1的email所有权已经移动到user2, 因此此处访问会出错。
    println!("Email:{}", user2.email);

}