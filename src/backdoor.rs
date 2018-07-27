use rug::{Assign, Integer};
use std::rc::Rc;
use std::sync::mpsc;
use pancurses::Window;
use time::precise_time_s;
use points::CurvePoint;
use prng::DualECDRBG;
use math::ModExtensions;

use num_cpus::get as num_cpus_get;
use crossbeam::scope as crossbeam_scope;

macro_rules! try_and_discard {
    ($e:expr) => (match $e {
        Ok(_) => (),
        Err(_) => ()
    });
}

lazy_static! {
    static ref THREE : Integer = Integer::from(3);
}

pub fn predict(prng : &DualECDRBG, d : &Integer, output1 : &Integer, output2 : &Integer, window : &Window) -> Option<Integer> {
    crossbeam_scope(|scope| {
        let (tx, rx) = mpsc::channel();
        let num_threads = num_cpus_get();

        window.printw(format!("Recovering lost bits using {} threads ...\n", num_threads));
        window.refresh();

        for thread_id in 0..num_threads {
            let tx = tx.clone(); 
            let output1 = output1.clone();
            scope.spawn(move || {
                let curve = Rc::new(prng.curve.clone());
                let mut sent = false;
                let mut prefix = thread_id;

                let mut rqy2 = Integer::new();
                let mut buffer = Integer::new();

                let q = prng.q.convert(Rc::clone(&curve)); 

                while prefix < 65536 {
                    let timestamp = precise_time_s();

                    buffer.assign(prefix);
                    buffer <<= prng.outsize;
                    let rqx = Integer::from(&buffer | &output1);

                    rqy2.assign(&rqx * &rqx);
                    rqy2 *= &rqx;
                    buffer.assign(&curve.a * &rqx);
                    rqy2 += &buffer;
                    rqy2 += &curve.b;
                    rqy2.modulo_mut(&curve.p);

                    let result : Option<Integer>;
                    if curve.name == "P-256" { 
                        result = rqy2.p256_mod_sqrt();
                    } 
                    else { 
                        result = rqy2.mod_sqrt(&curve.p); 
                    } 

                    match result {
                        Some(rqy) => {
                            let rq = CurvePoint {
                                x: rqx,
                                y: rqy,
                                curve: Rc::clone(&curve)
                            };

                            let state_guess = (&rq * d).x;
                            let output2_guess = (&q * &state_guess).x & &prng.outmask;

                            if &output2_guess == output2 {
                                try_and_discard!(tx.send(
                                        (false, 
                                         None, 
                                         format!("{:4x} ({:5}) | Found: {}\n", prefix, prefix, state_guess.to_string_radix(16)))
                                ));
                                try_and_discard!(tx.send(
                                        (true, 
                                         Some(state_guess), 
                                         "".to_string())
                                ));
                                sent = true;
                                break;
                            }
                        },
                        None => () 
                    }

                    let time_used = (precise_time_s() - timestamp) * 1000.0;
                    try_and_discard!(tx.send(
                            (false, 
                             None, 
                             format!("{:4x} ({:5}) | Took {} ms\n", prefix, prefix, time_used))
                    ));

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


