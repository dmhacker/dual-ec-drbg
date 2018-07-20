use num::bigint::BigInt;

pub struct Curve {
    name: String,   // Name of the curve
    bitsize: u16,   // Level of security offered by the curve 
    p: BigInt,      // Order of the finite field that the curve belongs to (mod p)
    n: BigInt,      // Order of the elliptic curve group
    a: BigInt,      // a parameter in the curve equation
    b: BigInt,      // b parameter in the curve equation
    gx: BigInt,     // X coordinate of generator (base) point
    gy: BigInt      // Y coordinate of generator (base) point
}
