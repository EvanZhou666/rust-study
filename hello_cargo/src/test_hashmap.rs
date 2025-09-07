use std::collections::HashMap;

fn main() {
    // 创建HashMap
    let mut hashmap = HashMap::new();

    // 插入值
    hashmap.insert(String::from("name"), String::from("张三"));

    println!("{:?}", hashmap.get("name"));

    // 不存在，则更新
    hashmap.entry(String::from("age")).or_insert(String::from("18")); // rust hashmap value要支持多种类型有点复杂，后面再学
    println!("{:?}", hashmap.get("age"));

    println!("=== 迭代键值对 ===");
    for (k, v) in &hashmap {
        println!("{}: {}", k, v);
    }
}