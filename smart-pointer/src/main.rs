use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    /**********************Rc用法******************************/
/*    let rc_ref = Rc::new(5);
    /*共享所有权，增加引用计数*/
    let a = rc_ref.clone(); // 引用个数+1
    let b = rc_ref.clone();// 引用个数+1
    println!("{:?}", a);
    println!("{:?}", b);
    println!("引用个数={:?}", Rc::strong_count(&rc_ref)); // 3*/


    /**********************RecCell用法******************************/
/*    let ref_cell = RefCell::new(5);
    {
        println!("可多次借用不可变所有权!");
        let x = ref_cell.borrow();
        let y = ref_cell.borrow();
    }

    /* 借用可变所有权，修改数据*/
    {
        println!("借用可变所有权");
        let mut ref_mut = ref_cell.borrow_mut();
        *ref_mut += 10;
    }
    println!("{:?}", ref_cell.borrow());*/

    /**********************单线程王炸Rc + RecCell用法******************************/
    let rc = Rc::new(RefCell::new(100));
    let a = rc.clone();
    let b= rc.clone();
    *a.borrow_mut() += 1;
    *b.borrow_mut() += 1;
    println!("{}", *rc.borrow()); // 102
}

