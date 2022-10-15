use std::thread;
/* Why It Gets Worse: Shared Data */
// To understand why this happens, we must understand the code sequence that the compiler generates for the update to counter. 
static mut counter: isize = 0;

fn my_thread(arg: &str) {
    println!("{}: begin", arg);
    for _ in 0..10_usize.pow(7) {
        unsafe {
            counter += 1;
        }
    }
    
    println!("{}: done", arg);
}

fn main() {
    unsafe {
        println!("main: begin (counter = {})", counter);
    }

    let t1 = thread::spawn(|| {
        my_thread("A");
    });

    let t2 = thread::spawn(|| {
        my_thread("B");
    });

    t1.join().unwrap();
    t2.join().unwrap();

    unsafe {
        println!("main: done with both (counter = {})", counter);
    }
}
/*
 * By using this hardware support, in combination with some help from the operating system, 
 * we will be able to build multi-threaded code that accesses critical sections in a synchronized and controlled manner, 
 * and thus reliably produces the correct result despite the challenging nature of concurrent execution. 
 */

