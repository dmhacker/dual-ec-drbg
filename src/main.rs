extern crate num;
extern crate rand;
extern crate crossbeam;

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
use std::sync::mpsc;

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
    let num_threads = 8;
    let (tx, rx) = mpsc::channel();
    let curve = &prng.curve;
    crossbeam::scope(|scope| {
        for thread_id in 0..num_threads {
            let tx = mpsc::Sender::clone(&tx);    
            let job_queue : Vec<usize> = (0..65536).filter(|&j| j % num_threads == thread_id).collect();
            scope.spawn(move || {
                let two = BigInt::from(2);
                let mut sent = false;
                for prefix in job_queue {
                    let adjusted_prefix = ToBigInt::to_bigint(&prefix).unwrap() << output1.bits();
                    let rqx = &adjusted_prefix | output1;
                    let rqy2 = (&rqx * &rqx * &rqx + &curve.a * &rqx + &curve.b).modpow(&One::one(), &curve.p);
                    println!("{} | {} | {}", prefix, adjusted_prefix.to_str_radix(16), rqx.to_str_radix(16));
                    match tonelli_shanks(&rqy2, &curve.p) {
                        Some(rqy) => {
                            if &rqy2 == &rqy.modpow(&two, &curve.p) {
                                let rq = CurvePoint {
                                    x: rqx,
                                    y: rqy
                                };
                                let state_guess = curve.multiply(&rq, d).x;
                                let output2_guess = curve.multiply(&prng.q, &state_guess).x & &prng.bitmask;
                                if &output2_guess == output2 {
                                    tx.send(Some(state_guess)).unwrap();
                                    sent = true;
                                    break;
                                }
                            }
                        },
                        None => () 
                    }
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

