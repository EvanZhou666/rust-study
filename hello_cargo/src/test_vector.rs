#[derive(Debug)]
#[allow(dead_code)]
enum SpreadsheetCell {
        Int(i32),
        Float(f64),
        Text(String),
}


fn main() {

    // 创建i32向量
    let mut v: Vec<i32> = Vec::new();
    // 添加元素
    v.push(1);
    v.push(2);
    v.push(3);

    for iv in &v{ // 注意：迭代向量的时候，v的所有权也会发生移动，所以这里使用&借用所有权
        println!("{}", iv)
    }

    // 创建向量的方式2，使用宏自动推断类型
    let v = vec![4,5,6];

    for iv in &v {
        println!("{}", iv);
    }

    // 获取向量中的元素
    // 方式1：
    println!("第1个元素是{}", v[1]);

    // 方式2：
    let option:Option<&i32> = v.get(2); // get返回的是Option枚举
    match option {
        None => println!("无法获取第2个元素"),
        Some(ii) => println!("第2个元素是{}", ii),
    }

    if let Some(value) = option {
        println!("也可以使用if语句获取Option的值{}", value); // 也可以使用if语句获取Option的值6
    }

    // 使用向量存储不同的类型

    let row = vec![
        SpreadsheetCell::Int(3),
        SpreadsheetCell::Text(String::from("blue")),
        SpreadsheetCell::Float(10.12),
    ];

    println!("{:?}", row) // [Int(3), Text("blue"), Float(10.12)]

}