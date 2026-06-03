use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取当前目录或命令行参数
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());

    // 读取目录
    let entries = fs::read_dir(Path::new(&path))?;

    // 打印文件名
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        println!("{}", file_name.to_string_lossy());
    }

    Ok(())
}
