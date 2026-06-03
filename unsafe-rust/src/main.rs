fn main() {
    let mut num =5;
    // 不可变裸指针(只读裸指针)
    let a = &raw const num;
    // 可变裸指针
    let b = &raw mut num;
    unsafe { // 访问不安全代码
        *b +=1;
        println!("{} and {}", *a, *b);
    }

/*    let p: *const i32;

    {
        let x = 10;
        p = &x as *const i32;
    } // x 已经被释放

    unsafe {
        println!("{}", *p); // ❌ UB
    }*/
}
