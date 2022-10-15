use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let n: u64  = rng.gen();
    println!("{}", n);
}
