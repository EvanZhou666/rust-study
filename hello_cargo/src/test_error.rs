use std::fs::File;
use std::io::ErrorKind;
fn main() {

    #[allow(unused_variables)]
    #[allow(non_snake_case)]
    let errorEnu = File::open("hello.text"); // 返回Error枚举
    // 使用match表达式处理异常
    // match errorEnu {
    //     Ok(f) => println!("查找到文件名hello.txt{:?}", f),
    //     _other_error => panic!("文件不存在"),
    // }

    // errorEnu.unwrap(); // 如果没有没有，返回Ok枚举的值，否则panic出异常

    // errorEnu.expect("hello.text文件不存在"); // 使用自定义的信息panic出异常

    // 使用unwrap_or_else闭包处理异常
    errorEnu.unwrap_or_else(|error|{
        if error.kind() == ErrorKind::NotFound {
            File::create("hello.txt").unwrap_or_else(|error| {
                panic!("创建hello.txt文件失败{}", error)
            })
        } else {
             panic!("{}", "Problem opening the file: {error:?}");
        }

    });

}