extern crate num;
extern crate rand;

pub mod math;
pub mod points;
pub mod curves;
pub mod prng;

use curves::Curve;
use points::CurvePoint;
use num::bigint::{BigInt, ToBigInt};
use num::traits::One;
use rand::prelude::random;
use prng::DualECDRBG;
use math::{mod_inverse, tonelli_shanks};

fn main() {
    let curve = Curve::gen_p256();
    println!("Curve = {}", curve.name);

    let seed_u64 : u64 = random();
    let seed = &ToBigInt::to_bigint(&seed_u64).unwrap(); 
    println!("Seed (s) = {}", seed);

    let backdoor = BigInt::parse_bytes(b"10", 10).unwrap();
    println!("Backdoor (e) = {}", backdoor);

    let q = curve.multiply(&curve.g, &mod_inverse(&backdoor, &curve.n).unwrap());
    println!("P = {}", curve.g);
    println!("Q = {}", q);
    println!("eQ = {}", curve.multiply(&q, &backdoor));

    let mut prng = DualECDRBG::new(&curve, &seed, &curve.g, &q);
  
    let output = prng.next();
    println!("\nEve observed output {}.", output);

    let mut prefix : BigInt = prng.prefix.clone();
    let mut rq = CurvePoint::origin(); 
    let mut rq_found = false;
    while prefix < BigInt::from(65536) {
        let rqx = (ToBigInt::to_bigint(&prefix).unwrap() << output.bits()) + &output;
        let ysq = (&rqx * &rqx * &rqx + &curve.a * &rqx + &curve.b).modpow(&One::one(), &curve.p);
        match tonelli_shanks(&ysq, &curve.p) {
            Some(rqy) => {
                if &ysq == &rqy.modpow(&BigInt::from(2), &curve.p) {
                    rq = CurvePoint {
                        x: rqx,
                        y: rqy
                    };
                    rq_found = true;
                    break;
                }
            },
            None => println!("Prefix {} failed.", prefix)
        }
        prefix += 1
    }

    if rq_found {
        let s = curve.multiply(&rq, &backdoor);
        println!("Eve guessed state {}.", s.x);
        println!("Actual state is {}.", &prng.s);
    } 
    else {
        println!("Eve was not able to guess the state this time.");
    }
}

#[cfg(test)]
mod tests {
    use num::bigint::BigInt;
    use math::{mod_inverse, prime_mod_inverse};

    #[test]
    fn test_positive_mod_inverse() {
        let inverse = mod_inverse(&BigInt::parse_bytes(b"4", 10).unwrap(), &BigInt::parse_bytes(b"7", 10).unwrap()).unwrap();
        assert_eq!(inverse, BigInt::parse_bytes(b"2", 10).unwrap());
    }

    #[test]
    fn test_negative_mod_inverse() {
        let inverse = mod_inverse(&BigInt::parse_bytes(b"-4", 10).unwrap(), &BigInt::parse_bytes(b"7", 10).unwrap()).unwrap();
        assert_eq!(inverse, BigInt::parse_bytes(b"5", 10).unwrap());
    }

    #[test]
    fn test_positive_prime_mod_inverse() {
        let inverse = prime_mod_inverse(&BigInt::parse_bytes(b"4", 10).unwrap(), &BigInt::parse_bytes(b"7", 10).unwrap());
        assert_eq!(inverse, BigInt::parse_bytes(b"2", 10).unwrap());
    }

    #[test]
    fn test_negative_prime_mod_inverse() {
        let inverse = prime_mod_inverse(&BigInt::parse_bytes(b"-4", 10).unwrap(), &BigInt::parse_bytes(b"7", 10).unwrap());
        assert_eq!(inverse, BigInt::parse_bytes(b"5", 10).unwrap());
    }

}
