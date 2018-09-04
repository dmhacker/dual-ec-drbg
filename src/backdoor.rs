use rug::{Assign, Integer};
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
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

pub fn predict(prng : &DualECDRBG, d : &Integer, output : &Integer, debug : bool) -> Option<Integer> {
    crossbeam_scope(|scope| {
        let (tx, rx) = mpsc::channel();
        let num_threads = num_cpus_get();
        let halt_counter = Arc::new(Mutex::new(false));

        // Print debug information title
        if debug {
            println!("\nDebug information:");
        }

        for thread_id in 0..num_threads {
            let tx = tx.clone(); 
            let halt_counter = halt_counter.clone();

            scope.spawn(move || {
                // This is set to true if the thread has sent a work result to the main thread
                let mut sent = false;

                // Closure for sending work result to main thread
                let send_result = |work_result: Option<Integer>, sent_counter: &mut bool| {
                    try_and_discard!(tx.send(
                        (work_result, "".to_string())
                    ));
                    *sent_counter = true;
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

                // The first DRBG output is the upper `outsize` bits without the last 16 bits
                let output1 = Integer::from(output >> 16);

                // Compute part of output 2's mask
                let mut output2_mask = Integer::from(Integer::u_pow_u(2, 16));
                output2_mask -= 1;

                // The second DRBG output constitutes the lower 16 bits
                let output2 = Integer::from(output & &output2_mask);

                // Transform the mask for later use
                output2_mask <<= prng.outsize - 16;

                while prefix < 65536 {
                    // Closure for sending debug message to main thread
                    let message_prefix = format!("[Thread {}] [Prefix {:04x} ({:05})]", thread_id, prefix, prefix);
                    let send_message = |debug_message: String| {
                        try_and_discard!(tx.send(
                            (None, format!("{}: {}      ", message_prefix, debug_message))
                        ));
                    };

                    // Measure time for each prefix computation
                    let timestamp = precise_time_s();

                    // Check if we should halt by unlocking the guarded global counter
                    {
                        let halt = halt_counter.lock().unwrap();
                        if *halt {
                            break;
                        }
                    }

                    // Determine rQ.x by adding back the lost bits
                    buffer.assign(prefix);
                    buffer <<= prng.outsize;
                    let rqx = Integer::from(&buffer | &output1);

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
                            let mut output2_guess = (&q * &state_guess).x & &prng.outmask;
                            output2_guess &= &output2_mask;
                            output2_guess >>= &prng.outsize - 16;

                            // Check to make sure the outputs match up
                            if output2_guess == output2 {
                                send_message(format!("Time = {0:.3} ms", (precise_time_s() - timestamp) * 1000.0));
                                send_result(Some(state_guess), &mut sent);
                                break;
                            }
                        },
                        None => () 
                    }

                    send_message(format!("Time = {0:.3} ms", (precise_time_s() - timestamp) * 1000.0));
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

                                // Any other active child threads should halt 
                                let mut halt = halt_counter.lock().unwrap();
                                *halt = true;
                            },
                            None => () 
                        }

                        // If a thread submitted a work result, it is no longer active
                        threads_finished += 1
                    }
                    else if debug && global_result.is_none() {
                        print!("\r{}", message);
                    }
                },
                _ => ()
            }
        }

        // Print first newline so that previous carriage return is completed
        // Print additional newline that future output appears separately
        if debug {
            println!("\n");
        }

        global_result 
    })
}


