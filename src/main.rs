extern crate ramp;
extern crate rand;
extern crate crossbeam;
extern crate time;
extern crate num_cpus;
extern crate pancurses;

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
use pancurses::{initscr, endwin, Window};

fn main() {
    let window = initscr();
    
    let curve = Curve::gen_p256();
    let seed = rand::thread_rng().gen_uint(curve.bitsize); 
    let d = Int::from_str_radix("fffffffffffffff", 16).unwrap();
    let q = curve.multiply(&curve.g, &mod_inverse(&d, &curve.n).unwrap());
    let mut prng = DualECDRBG::new(&curve, &seed, &curve.g, &q);

    window.printw(format!("Curve = \t{}\n", curve.name));
    window.printw(format!("Seed = \t\t{}\n", seed.to_str_radix(16, false)));
    window.printw(format!("d = \t\t{}\n", d));
    window.printw(format!("Q = \t\t{}\n", q));
    window.printw(format!("dQ = \t\t{}\n", curve.multiply(&q, &d)));
    window.printw(format!("P = \t\t{}\n", curve.g));
    window.hline('-', 10000);
    window.mvprintw(window.get_cur_y() + 1, 0, "Alice is generating some output ...");
    window.refresh();
  
    let output1 = prng.next();
    let output2 = prng.next();

    window.deleteln();
    window.mvprintw(window.get_cur_y(), 0, format!("Alice generated output 1 {}.\n", output1.to_str_radix(16, false)));
    window.printw(format!("Alice generated output 2 {}.\n", output2.to_str_radix(16, false)));
    window.printw(format!("Eve has observed these outputs and will guess Alice's state.\n"));
    window.refresh();

    let (my, mx) = window.get_max_yx();
    let (cy, cx) = window.get_cur_yx();

    let nlines = (my - cy) / 2 - 1;
    let begy = (my + cy) / 2;
    window.mv(begy - 1, cx);
    window.hline('-', 10000);
    window.mv(cy, cx);
    window.refresh();

    let subwindow = window.derwin(nlines, mx, begy, cx).unwrap();
    subwindow.setscrreg(0, nlines - 1);
    subwindow.scrollok(true);

    match predict(&prng, &d, &output1, &output2, &subwindow) {
        Some(state) => {
            window.printw(format!("Eve guessed state {}.\n", &state));
            window.printw(format!("Alice's actual state is {}.\n", &prng.s));
        },
        None => {
            window.printw(format!("Eve was not able to guess Alice's state this time.\n")) ;
        }
    } 

    window.getch();
    endwin();
}

fn predict(prng : &DualECDRBG, d : &Int, output1 : &Int, output2 : &Int, window : &Window) -> Option<Int> {
    let (tx, rx) = mpsc::channel();
    let num_threads = num_cpus::get();

    window.printw(format!("Recovering lost bits using {} threads ...\n", num_threads));
    window.refresh();

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

                            tx.send((false, None, format!("{} | State guess was {}\n", prefix, state_guess.to_str_radix(16, false)))).unwrap();
                            tx.send((false, None, format!("{} | Output guess was {}\n", prefix, output2_guess.to_str_radix(16, false)))).unwrap();
                            tx.send((false, None, format!("{} | Output truth was {}\n", prefix, output2.to_str_radix(16, false)))).unwrap();

                            if &output2_guess == output2 {
                                tx.send((true, Some(state_guess), "".to_string())).unwrap();
                                sent = true;
                                break;
                            }
                        },
                        None => () 
                    }

                    tx.send((false, None, format!("{} | Took {} seconds\n", prefix, precise_time_s() - timestamp))).unwrap();

                    prefix += num_threads;
                }            
                if !sent {
                    tx.send((true, None, "".to_string())).unwrap(); 
                }
            });
        }
    });

    let mut threads_finished = 0;
    while threads_finished < num_threads {
        let (is_result, result, message) = rx.recv().unwrap();
        if is_result {
            match result {
                Some(ret) => return Some(ret),
                None => threads_finished += 1
            }
        }
        else {
            window.printw(message);
            window.refresh();
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

