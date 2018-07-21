extern crate num;

pub mod math;
pub mod points;
pub mod curves;

use curves::Curve;
use num::bigint::ToBigInt;

fn main() {
    let curve = Curve::gen_p256();
    let point1 = curve.multiply(&curve.g, &ToBigInt::to_bigint(&5).unwrap()); 
    println!("{}", point1);
}
