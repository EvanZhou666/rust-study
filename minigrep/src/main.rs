use std::{env, fs};

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    dbg!(&args); // 打印向量参数
    let query = &args[1];
    let filename = &args[2];
    println!("查询的文件名是{}, 字符串是{}", filename, query);
    let fs_content = fs::read_to_string(filename).expect("文件不存在");
    // println!("{}", fs_content);
    let mut result = vec![];
    for line in fs_content.lines() {
        if line.contains(query) {
            // println!("{}", line);
            result.push(line);
        }
    }

    println!("{:?}", result);

}











