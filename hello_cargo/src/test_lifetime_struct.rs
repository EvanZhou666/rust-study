#[derive(Debug)]
struct Order<'s> {
    order_id: i64,
    amount: i32,
    product: &'s Product<'s>, // 说明order实例的生命周期不能超过内部product引用的生命周期
}

#[derive(Debug)]
struct Product<'a> {
    product_id: i32,
    product_name: &'a str, // product结构体实例的生命周期不超过内部切片product_name的生命周期
}

fn main() {

    let name =  "iPhone17 pro";
    let product = Product{
        product_id: 1,
        product_name: name
    };


    let order = Order{
        order_id: 20240906001,
        amount: 99,
        product: &product,
    };


    println!("{:?}", order);
    println!("{:?}", product);
}