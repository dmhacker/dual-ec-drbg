extern crate num;
extern crate rand;
extern crate crossbeam;
extern crate time;

pub mod math;
pub mod points;
pub mod curves;
pub mod prng;

use curves::Curve;
use points::CurvePoint;
use num::bigint::{BigInt, ToBigInt};
use rand::prelude::random;
use prng::DualECDRBG;
use math::{mod_inverse, mod_sqrt, p256_mod_sqrt, two_pow, modulo};
use std::sync::mpsc;
use time::precise_time_s;

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
    
    match predict(&prng, &d, &output1, &output2) {
        Some(state) => {
            println!("Eve guessed state {}.", &state);
            println!("Actual state is {}.", &prng.s);
        },
        None => println!("Eve was not able to guess the state this time.")
    } 
}

fn predict(prng : &DualECDRBG, d : &BigInt, output1 : &BigInt, output2: &BigInt) -> Option<BigInt> {
    let (tx, rx) = mpsc::channel();
    let num_threads = 8;
    crossbeam::scope(|scope| {
        for thread_id in 0..num_threads {
            let tx = mpsc::Sender::clone(&tx);    
            scope.spawn(move || {
                let curve = &prng.curve;
                let bitmask = two_pow(curve.bitsize - 16) - 1;
                let mut sent = false;
                let mut prefix = thread_id;
                while prefix < 65536 {
                    let adjusted_prefix = ToBigInt::to_bigint(&prefix).unwrap() << output1.bits();
                    let rqx = adjusted_prefix | output1;
                    let rqy2 = modulo(&(&rqx * &rqx * &rqx + &curve.a * &rqx + &curve.b), &curve.p);
                    let result : Option<BigInt>;
                    let timestamp = precise_time_s();
                    if curve.name == "P-256" {
                        result = p256_mod_sqrt(&rqy2, &curve.p);
                    }
                    else {
                        result = mod_sqrt(&rqy2, &curve.p);
                    }
                    let ms = (precise_time_s() - timestamp) * 1000.0;
                    match result {
                        Some(rqy) => {
                            println!("{} | mod_sqrt took {} ms", prefix, ms);
                            let rq = CurvePoint {
                                x: rqx,
                                y: rqy
                            };
                            let state_guess = curve.multiply(&rq, d).x;
                            let output2_guess = curve.multiply(&prng.q, &state_guess).x & &bitmask;
                            if &output2_guess == output2 {
                                tx.send(Some(state_guess)).unwrap();
                                sent = true;
                                break;
                            }
                        },
                        None => () 
                    }
                    prefix += num_threads;
                }            
                if !sent {
                    tx.send(None).unwrap(); 
                }
            });
        }
    });

    for _ in 0..num_threads {
        match rx.recv().unwrap() {
            Some(result) => return Some(result),
            None => ()
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use num::bigint::BigInt;
    use math::{mod_inverse, prime_mod_inverse, two_pow};

    #[test]
    fn test_positive_mod_inverse() {
        let inverse = mod_inverse(&BigInt::parse_bytes(b"4", 10).unwrap(), &BigInt::parse_bytes(b"7", 10).unwrap());
        assert_eq!(inverse.unwrap(), BigInt::parse_bytes(b"2", 10).unwrap());
    }

    #[test]
    fn test_negative_mod_inverse() {
        let inverse = mod_inverse(&BigInt::parse_bytes(b"-4", 10).unwrap(), &BigInt::parse_bytes(b"7", 10).unwrap());
        assert_eq!(inverse.unwrap(), BigInt::parse_bytes(b"5", 10).unwrap());
    }

    #[test]
    fn test_positive_prime_mod_inverse() {
        let inverse = prime_mod_inverse(&BigInt::parse_bytes(b"4", 10).unwrap(), &BigInt::parse_bytes(b"7", 10).unwrap());
        assert_eq!(inverse.unwrap(), BigInt::parse_bytes(b"2", 10).unwrap());
    }

    #[test]
    fn test_negative_prime_mod_inverse() {
        let inverse = prime_mod_inverse(&BigInt::parse_bytes(b"-4", 10).unwrap(), &BigInt::parse_bytes(b"7", 10).unwrap());
        assert_eq!(inverse.unwrap(), BigInt::parse_bytes(b"5", 10).unwrap());
    }

    #[test]
    fn test_two_pow_one() {
        assert_eq!(BigInt::from(2), two_pow(1));
    }

    #[test]
    fn test_two_pow_ten() {
        assert_eq!(BigInt::from(1024), two_pow(10));
    }
}

