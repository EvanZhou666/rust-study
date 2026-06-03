use std::fs;

fn main() {
    let read_dir = fs::read_dir("./").unwrap();
    for dir_entry in read_dir {
        let dir_entry = dir_entry.unwrap();
        let is_dir = dir_entry.file_type().unwrap().is_dir();
        let file_size = dir_entry.metadata().unwrap().len();
        println!("{}--{}----{}", if is_dir {"dr"} else {"f"},  dir_entry.file_name().to_str().unwrap(), file_size);
    }

}
