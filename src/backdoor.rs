use rug::{Assign, Integer};
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
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

        let global_finished = Arc::new(Mutex::new(false));

        for thread_id in 0..num_threads {
            let tx = tx.clone(); 
            let global_finished = global_finished.clone();
            scope.spawn(move || {
                let send_result = |work_result: Option<Integer>| try_and_discard!(tx.send(
                    (work_result, "".to_string())
                ));

                let curve = Rc::new(prng.curve.clone());
                let mut sent = false;
                let mut prefix = thread_id;

                let mut rqy2 = Integer::new();
                let mut buffer = Integer::new();

                let q = prng.q.convert(Rc::clone(&curve)); 

                while prefix < 65536 {
                    let message_prefix = format!("[{}] [{:04x} ({:05})]", thread_id, prefix, prefix);
                    let send_message = |debug_message: String| try_and_discard!(tx.send(
                        (None, format!("{}: {}\n", message_prefix, debug_message))
                    ));

                    let timestamp = precise_time_s();

                    {
                        let halt = global_finished.lock().unwrap();
                        if *halt {
                            break;
                        }
                    }

                    buffer.assign(prefix);
                    buffer <<= prng.outsize;
                    let rqx = Integer::from(&buffer | output1);

                    rqy2.assign(&rqx * &rqx);
                    rqy2 *= &rqx;
                    buffer.assign(&curve.a * &rqx);
                    rqy2 += &buffer;
                    rqy2 += &curve.b;
                    rqy2.modulo_mut(&curve.p);

                    match rqy2.sqrt_mod(&curve.p) {
                        Some(rqy) => {
                            let rq = CurvePoint {
                                x: rqx,
                                y: rqy,
                                curve: Rc::clone(&curve)
                            };

                            let state_guess = (&rq * d).x;
                            let output2_guess = (&q * &state_guess).x & &prng.outmask;

                            if &output2_guess == output2 {
                                send_message(format!("rQ = {}", rq));
                                send_message(format!("drQ.x = {}", state_guess.to_string_radix(16)));
                                send_result(Some(state_guess));
                                sent = true;
                                break;
                            }
                        },
                        None => () 
                    }

                    let time_used = (precise_time_s() - timestamp) * 1000.0;
                    send_message(format!("Took {} ms", time_used));
                    prefix += num_threads;
                }            

                if !sent {
                    send_result(None);
                }
            });
        }

        let mut global_result = None;
        let mut threads_finished = 0;
        while threads_finished < num_threads {
            match rx.recv() {
                Ok((result, message)) => {
                    if message == "" {
                        match result {
                            Some(ret) => {
                                global_result = Some(ret);
                                let mut halt = global_finished.lock().unwrap();
                                *halt = true;
                            },
                            None => () 
                        }
                        threads_finished += 1
                    }
                    else if global_result.is_none() {
                        window.printw(message);
                        window.refresh();
                    }
                },
                _ => ()
            }
        }

        global_result 
    })
}


