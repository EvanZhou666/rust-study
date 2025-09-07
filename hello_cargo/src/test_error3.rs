use std::fs::File;
use std::io::{self, Read};

fn read_username_from_file() -> Result<String, io::Error> {
    // 如果返回值是OK,?操作符会返回OK枚举中的值，如果是error，整个函数会返回该error
    let mut username_file = File::open("hello.txt")?;
    let mut username = String::new();
    username_file.read_to_string(&mut username)?;
    Ok(username)
}

fn main() {
    read_username_from_file();
}
