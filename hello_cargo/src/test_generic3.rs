fn main() {
    let r;

    {
        let x = 5;
        // 编译会出错，r借用了x的所有权，当x超出作用域的时候，x会被drop，所有权将不存在
        // r = &x;
        // 改为
        r = x;
    }

    println!("r: {r}");
}