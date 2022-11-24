use serde::{Serialize, Deserialize};

// #[derive(Serialize, Deserialize)]
#[derive(Serialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize)]
pub struct Triple {
    xy: Point, 
    z: i32
}

#[derive(Serialize)]
pub struct Seq {
    pub arr: Vec<u32>,
    pub s: String,
}

fn main() {
    let point = Point { x: 1, y: 1 };
    let se = serde_json::to_string(&point).unwrap();   
    println!("{}", se);
    let sep = serde_json::to_string_pretty(&point).unwrap();   
    println!("{}", sep);
    
    let triple = Triple { xy: point, z: 2 };
    let se = serde_json::to_string(&triple).unwrap();   
    println!("{}", se);
    
    // sequence
    let seq = Seq { arr: vec![0, 1, 2, 3], s: String::from("hello") };
    let se = serde_json::to_string(&seq).unwrap();   
    println!("{}", se);
}