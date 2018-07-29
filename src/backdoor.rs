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
    // Wrap in a crossbeam scope, so that we can use clones of the certain thread-safe formal parameters
    crossbeam_scope(|scope| {
        // Open a channel for communicating debug messages & work results
        let (tx, rx) = mpsc::channel();

        // Use the same number of threads as processors on the machine
        let num_threads = num_cpus_get();

        // Use a global counter to tell child threads to halt computation
        let global_finished = Arc::new(Mutex::new(false));

        for thread_id in 0..num_threads {
            // Clone references to channel and halt counter
            let tx = tx.clone(); 
            let global_finished = global_finished.clone();

            scope.spawn(move || {
                // This is set to true if the thread has sent a work result to the main thread
                let mut sent = false;

                // Closure for sending work result to main thread
                let send_result = |work_result: Option<Integer>, sent_counter: &mut bool| {
                    try_and_discard!(tx.send(
                        (work_result, "".to_string())
                    ));
                    *sent_counter = false;
                };

                // Each thread has its own version of the curve and Q
                // This way, we don't have to use slow atomic references to access the same area of memory
                let curve = Rc::new(prng.curve.clone());
                let q = CurvePoint::from(&prng.q, &curve); 

                // Thread is responsible for all lost bits `prefix` such that prefix = thread_id mod num_threads
                let mut prefix = thread_id;

                // Integer buffers for use in computations
                let mut rqy2 = Integer::new();
                let mut buffer = Integer::new();

                while prefix < 65536 {
                    // Closure for sending debug message to main thread
                    let message_prefix = format!("[{}] [{:04x} ({:05})]", thread_id, prefix, prefix);
                    let send_message = |debug_message: String| {
                        try_and_discard!(tx.send(
                            (None, format!("{}: {}\n", message_prefix, debug_message))
                        ));
                    };

                    // Measure time for each prefix computation
                    let timestamp = precise_time_s();

                    // Check if we should halt by unlocking the guarded global counter
                    {
                        let halt = global_finished.lock().unwrap();
                        if *halt {
                            break;
                        }
                    }

                    // Determine rQ.x by adding back the lost bits
                    buffer.assign(prefix);
                    buffer <<= prng.outsize;
                    let rqx = Integer::from(&buffer | output1);

                    // Calculate (rQ.y)^2 using the curve equation
                    rqy2.assign(&rqx * &rqx);
                    rqy2 *= &rqx;
                    buffer.assign(&curve.a * &rqx);
                    rqy2 += &buffer;
                    rqy2 += &curve.b;
                    rqy2.modulo_mut(&curve.p);

                    // Determine rQ.y and make sure it lies on the curve
                    match rqy2.sqrt_mod(&curve.p) {
                        Some(rqy) => {
                            // Now that we know rQ's x, y coordinates, we know the point itself
                            let rq = CurvePoint {
                                x: rqx,
                                y: rqy,
                                curve: Rc::clone(&curve)
                            };

                            // The current state of the PRNG equals rP = r(dQ) = d(rQ)
                            let state_guess = (&rq * d).x;

                            // Now that we have the state, we can make a guess as to the second output of the PRNG
                            let output2_guess = (&q * &state_guess).x & &prng.outmask;

                            // Check to make sure the outputs match up
                            if &output2_guess == output2 {
                                send_message(format!("rQ = {}", rq));
                                send_message(format!("drQ.x = {}", state_guess.to_string_radix(16)));
                                send_message(format!("Time = {:.3} ms", (precise_time_s() - timestamp) * 1000.0));
                                send_result(Some(state_guess), &mut sent);
                                break;
                            }
                        },
                        None => () 
                    }

                    send_message(format!("Time = {:.3} ms", (precise_time_s() - timestamp) * 1000.0));
                    prefix += num_threads;
                }            

                // If this thread found nothing, send an empty work result to the main thread
                if !sent {
                    send_result(None, &mut sent);
                }
            });
        }

        // Contains the predicted state (or nothing)
        let mut global_result = None;

        // Keeps track of how many child threads are inactive
        let mut threads_finished = 0;

        while threads_finished < num_threads {
            match rx.recv() {
                Ok((result, message)) => {
                    // An empty message indicates that a work result was submitted
                    if message == "" {
                        match result {
                            Some(ret) => {
                                // Set the global result to be that child's work result 
                                global_result = Some(ret);

                                // Mark that all child threads should halt 
                                let mut halt = global_finished.lock().unwrap();
                                *halt = true;
                            },
                            None => () 
                        }

                        // If a thread submitted a work result, it is no longer active
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


