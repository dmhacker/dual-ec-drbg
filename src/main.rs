extern crate ramp;
extern crate rand;
extern crate crossbeam;
extern crate time;
extern crate num_cpus;

pub mod math;
pub mod points;
pub mod curves;
pub mod prng;

use curves::Curve;
use points::CurvePoint;

use ramp::int::Int;
use ramp::RandomInt;
use prng::DualECDRBG;
use math::{mod_inverse, modulo, mod_sqrt, p256_mod_sqrt};
use std::sync::mpsc;
use time::precise_time_s;

fn main() {
    let curve = Curve::gen_p256();

    println!("Curve = {}", curve.name);

    let seed = rand::thread_rng().gen_uint(256); 

    println!("Seed = {}", seed);

    let d = Int::from_str_radix("10000000000000000000000000000", 10).unwrap();
    let q = curve.multiply(&curve.g, &mod_inverse(&d, &curve.n).unwrap());

    println!("d = {}", d);
    println!("Q = {}", q);
    println!("dQ = {}", curve.multiply(&q, &d));
    println!("P = {}\n", curve.g);

    let mut prng = DualECDRBG::new(&curve, &seed, &curve.g, &q);
  
    let output1 = prng.next();
    let output2 = prng.next();

    println!("Eve observed output 1 {}.", output1.to_str_radix(16, false));
    println!("Eve observed output 2 {}.", output2.to_str_radix(16, false));
    
    match predict(&prng, &d, &output1, &output2) {
        Some(state) => {
            println!("Eve guessed state {}.", &state);
            println!("Actual state is {}.", &prng.s);
        },
        None => println!("Eve was not able to guess the state this time.")
    } 
}

fn predict(prng : &DualECDRBG, d : &Int, output1 : &Int, output2: &Int) -> Option<Int> {
    let (tx, rx) = mpsc::channel();
    let num_threads = num_cpus::get();

    println!("Recovering lost bits using {} threads ...", num_threads);

    crossbeam::scope(|scope| {
        for thread_id in 0..num_threads {
            let tx = mpsc::Sender::clone(&tx);    
            scope.spawn(move || {
                let curve = &prng.curve;
                let bitmask = Int::from(2).pow(curve.bitsize - 16) - 1;
                let mut sent = false;
                let mut prefix = thread_id;
                while prefix < 65536 {
                    let timestamp = precise_time_s();

                    let lost_bits = Int::from(prefix) << (output1.bit_length() as usize);
                    let rqx = lost_bits | output1;
                    let rqy2 = modulo(&(&rqx * &rqx * &rqx + &curve.a * &rqx + &curve.b), &curve.p);
                    let result : Option<Int>;
                    if curve.name == "P-256" { 
                        result = p256_mod_sqrt(&rqy2);
                    } 
                    else { 
                        result = mod_sqrt(&rqy2, &curve.p); 
                    } 
                    match result {
                        Some(rqy) => {
                            let rq = CurvePoint {
                                x: rqx,
                                y: rqy
                            };

                            let state_guess = curve.multiply(&rq, d).x;
                            let output2_guess = curve.multiply(&prng.q, &state_guess).x & &bitmask; 

                            println!("{} | State guess was {}", prefix, state_guess.to_str_radix(16, false));
                            println!("{} | Output guess was {}", prefix, output2_guess.to_str_radix(16, false));
                            println!("{} | Output truth was {}", prefix, output2.to_str_radix(16, false));

                            if &output2_guess == output2 {
                                tx.send(Some(state_guess)).unwrap();
                                sent = true;
                                break;
                            }
                        },
                        None => () 
                    }

                    println!("{} | Took {} seconds", prefix, precise_time_s() - timestamp);
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
    use ramp::int::Int; 
    use math::{mod_inverse, prime_mod_inverse};

    #[test]
    fn test_positive_mod_inverse() {
        let inverse = mod_inverse(&Int::from(4), &Int::from(7));
        assert_eq!(inverse.unwrap(), Int::from(2));
    }

    #[test]
    fn test_negative_mod_inverse() {
        let inverse = mod_inverse(&Int::from(-4), &Int::from(7));
        assert_eq!(inverse.unwrap(), Int::from(5));
    }

    #[test]
    fn test_positive_prime_mod_inverse() {
        let inverse = prime_mod_inverse(&Int::from(4), &Int::from(7));
        assert_eq!(inverse.unwrap(), Int::from(2));
    }

    #[test]
    fn test_negative_prime_mod_inverse() {
        let inverse = prime_mod_inverse(&Int::from(-4), &Int::from(7));
        assert_eq!(inverse.unwrap(), Int::from(5));
    }
}

