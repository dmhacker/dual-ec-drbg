extern crate argparse;
extern crate crossbeam;
extern crate num_cpus;
extern crate rand;
extern crate rug;
#[macro_use]
extern crate lazy_static;

pub mod backdoor;
pub mod curves;
pub mod math;
pub mod points;
pub mod prng;

use argparse::{ArgumentParser, Store, StoreTrue};
use backdoor::predict;
use curves::Curve;
use math::RandExt;
use points::{CurvePoint, Point};
use prng::DualECDRBG;
use rug::Integer;
use std::rc::Rc;
use std::time::Instant;

fn main() {
    // Parse the command line arguments
    let mut curve_str = "P-256".to_string();
    let mut backdoor_str = "".to_string();
    let mut seed_str = "".to_string();
    let mut verbose = false;
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Interactive proof-of-concept of the Dual_EC_DRBG backdoor");
        parser.refer(&mut curve_str).add_option(
            &["--curve", "-c"],
            Store,
            "NIST-standard curve type, either P-256, P-384, or P-521",
        );
        parser.refer(&mut backdoor_str).add_option(
            &["--backdoor", "-b"],
            Store,
            "Backdoor to use (in decimal)",
        );
        parser.refer(&mut seed_str).add_option(
            &["--seed", "-s"],
            Store,
            "Seed to use (in decimal)",
        );
        parser.refer(&mut verbose).add_option(
            &["--verbose", "-v"],
            StoreTrue,
            "Print debug messages",
        );
        parser.parse_args_or_exit();
    }

    // Create a separate random number generator (for seed & backdoor values)
    let mut rng = rand::thread_rng();

    // Parse the curve arguments
    let curve: Rc<Curve>;
    if curve_str == "P-256" {
        curve = Rc::new(Curve::gen_p256());
    } else if curve_str == "P-384" {
        curve = Rc::new(Curve::gen_p384());
    } else if curve_str == "P-521" {
        curve = Rc::new(Curve::gen_p521());
    } else {
        eprintln!("Valid curves are P-256, P-384, P-521.");
        return;
    }

    // Parse the supplied backdoor argument or randomly generate it
    let d: Integer;
    if backdoor_str == "" {
        d = rng.gen_uint(curve.bitsize);
    } else {
        d = Integer::from_str_radix(&backdoor_str, 10).unwrap();
        if d < 2 as i32 {
            eprintln!("Backdoor must be greater than 2.");
            return;
        }
    }

    // Parse the supplied seed argument or randomly generate it
    let seed: Integer;
    if seed_str == "" {
        seed = rng.gen_uint(curve.bitsize);
    } else {
        seed = Integer::from_str_radix(&seed_str, 10).unwrap();
    }

    // Point P in the Dual_EC_DRBG is the curve's generator point as in NIST specifications
    let p = CurvePoint::from(&curve.g, &curve);

    // Generate point Q in the Dual_EC_DRBG algorithm using the backdoor
    let q = &p * &d.clone().invert(&curve.n).unwrap();

    // Use P, Q, the seed, and the curve to create the Dual_EC_DRBG PRNG
    let mut prng = DualECDRBG::new(&curve, &Point::from(&p), &Point::from(&q), &seed);

    // Create a curses window in the terminal for displaying all of this information
    println!("Curve = \t{}", curve.name);
    println!("Seed = \t\t{}", seed.to_string_radix(16));
    println!("d = \t\t{}", d.clone().to_string_radix(16));
    println!("Q = \t\t{}", q);
    println!("dQ = \t\t{}", &q * &d);
    println!("P = \t\t{}\n", p);

    // Generate and display two successive outputs from the DRBG
    let outsize = prng.outsize + 16;
    let output = prng.next_bits(outsize);
    println!(
        "Alice generated output {} ({} bytes).",
        output.to_string_radix(16),
        outsize / 8
    );
    println!("Eve has observed this output and will guess Alice's state.");

    // Do prediction and measure time it took
    let timestamp = Instant::now();
    let prediction = predict(&prng, &d, &output, verbose);
    println!(
        "Eve spent {:?} calculating Alice's state.",
        Instant::now() - timestamp
    );

    match prediction {
        Some(state) => {
            println!(
                "Eve guessed Alice's state as {}.",
                &state.to_string_radix(16)
            );
        }
        None => {
            println!("Eve was not able to guess Alice's state.");
        }
    }

    // Delegate state printing to the PRNG; the state variable is private so the predictor cannot cheat
    prng.print_state(&"Alice's actual state is ", &".");
}

#[cfg(test)]
mod tests {
    use crate::curves::Curve;
    use crate::math::RandExt;
    use crate::points::{CurvePoint, Point};
    use rand::thread_rng;
    use std::rc::Rc;

    #[test]
    fn test_point_multiplication() {
        let mut rng = thread_rng();
        let curve = Rc::new(Curve::gen_p256());
        let p = CurvePoint::from(&curve.g, &curve);
        for _ in 0..10 {
            // Compute Q = sP and make sure Q is on the same curve as P
            let s = rng.gen_uint(curve.bitsize);
            let q = &p * &s;
            assert!(Point::from(&q).is_on_curve(&curve),);

            // Compute P = (s^-1 mod n) * Q and make sure P is equal to the original P
            let i = s.invert(&curve.n).unwrap();
            let p1 = &q * &i;
            assert_eq!(p, p1);
        }
    }
}
