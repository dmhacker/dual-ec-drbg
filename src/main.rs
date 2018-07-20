extern crate num;

pub mod curves;

use curves::Curve;

fn main() {
    let p256 = Curve::gen_p256();
    println!("Hello, world!");
    println!("{} ... A = {}", p256.name, p256.a);
}
