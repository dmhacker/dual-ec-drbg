use ramp::int::Int;
use std::sync::mpsc;
use pancurses::Window;
use time::precise_time_s;
use points::CurvePoint;
use prng::DualECDRBG;
use math::{modulo, mod_sqrt, p256_mod_sqrt};

use num_cpus::get as num_cpus_get;
use crossbeam::scope as crossbeam_scope;

macro_rules! try_and_discard {
    ($e:expr) => (match $e {
        Ok(_) => (),
        Err(_) => ()
    });
}

pub fn predict(prng : &DualECDRBG, d : &Int, output1 : &Int, output2 : &Int, window : &Window) -> Option<Int> {
    crossbeam_scope(|scope| {
        let (tx, rx) = mpsc::channel();
        let num_threads = num_cpus_get();

        window.printw(format!("Recovering lost bits using {} threads ...\n", num_threads));
        window.refresh();

        for thread_id in 0..num_threads {
            let tx = tx.clone(); 
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

                            try_and_discard!(tx.send((false, None, format!("{} | State guess was {}\n", prefix, state_guess.to_str_radix(16, false)))));
                            try_and_discard!(tx.send((false, None, format!("{} | Output guess was {}\n", prefix, output2_guess.to_str_radix(16, false)))));
                            try_and_discard!(tx.send((false, None, format!("{} | Output truth was {}\n", prefix, output2.to_str_radix(16, false)))));

                            if &output2_guess == output2 {
                                try_and_discard!(tx.send((true, Some(state_guess), "".to_string())));
                                sent = true;
                                break;
                            }
                        },
                        None => () 
                    }

                    try_and_discard!(tx.send((false, None, format!("{} | Took {} seconds\n", prefix, precise_time_s() - timestamp))));

                    prefix += num_threads;
                }            
                if !sent {
                    try_and_discard!(tx.send((true, None, "".to_string())));
                }
            });
        }
        let mut threads_finished = 0;
        while threads_finished < num_threads {
            match rx.recv() {
                Ok((is_result, result, message)) => {
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
                },
                _ => ()
            }
        }
        None
    })
}


