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

    println!("Seed = {}", seed);

    let d = BigInt::parse_bytes(b"10", 10).unwrap();
    let q = curve.multiply(&curve.g, &mod_inverse(&d, &curve.n).unwrap());

    println!("d = {}", d);
    println!("Q = {}", q);
    println!("dQ = {}", curve.multiply(&q, &d));
    println!("P = {}\n", curve.g);

    let mut prng = DualECDRBG::new(&curve, &seed, &curve.g, &q);
  
    let output1 = prng.next();
    let output2 = prng.next();

    println!("Eve observed output 1 {}.", output1.to_str_radix(16));
    println!("Eve observed output 2 {}.", output2.to_str_radix(16));

    let mut state = BigInt::from(0); 
    let mut state_found = false;

    let two = BigInt::from(2);

    for prefix in 0..65536 {
        let rqx = (ToBigInt::to_bigint(&prefix).unwrap() << output1.bits()) | &output1;
        let rqy2 = (&rqx * &rqx * &rqx + &curve.a * &rqx + &curve.b).modpow(&One::one(), &curve.p);
        println!("{}| {}", prefix, rqx.to_str_radix(16));
        match tonelli_shanks(&rqy2, &curve.p) {
            Some(rqy) => {
                if &rqy2 == &rqy.modpow(&two, &curve.p) {
                    let rq = CurvePoint {
                        x: rqx,
                        y: rqy
                    };
                    let state_guess = curve.multiply(&rq, &d).x;
                    let output2_guess = curve.multiply(&q, &state_guess).x & &prng.bitmask;
                    if output2_guess == output2 {
                        state = state_guess;
                        state_found = true;
                        break;
                    }
                }
            },
            None => () 
        }
    }

    if state_found {
        println!("Eve guessed state {}.", &state);
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
