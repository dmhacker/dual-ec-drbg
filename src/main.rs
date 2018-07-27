extern crate ramp;
extern crate rand;
extern crate crossbeam;
extern crate time;
extern crate num_cpus;
extern crate pancurses;
extern crate argparse;
#[macro_use]
extern crate lazy_static;

pub mod math;
pub mod points;
pub mod curves;
pub mod prng;
pub mod backdoor;

use ramp::int::Int;
use ramp::RandomInt;
use curves::Curve;
use backdoor::predict;
use prng::DualECDRBG;
use argparse::{ArgumentParser, Store};
use math::ModExtensions;

fn main() {
    let mut curve_str = "P-256".to_string();
    let mut backdoor_str = "".to_string(); 
    let mut seed_str = "".to_string();
    {  
        let mut parser = ArgumentParser::new();
        parser.set_description("Interactive proof-of-concept of the Dual_EC_DRBG backdoor");
        parser.refer(&mut curve_str)
            .add_option(&["--curve", "-c"], Store,
            "NIST-standard curve type, either P-256, P-384, or P-521");
        parser.refer(&mut backdoor_str)
            .add_option(&["--backdoor", "-b"], Store,
            "Backdoor to use (in decimal)");
        parser.refer(&mut seed_str)
            .add_option(&["--seed", "-s"], Store,
            "Seed to use (in decimal)");
        parser.parse_args_or_exit();
    }

    let mut seed_rng = rand::thread_rng();

    let curve : Curve;
    if curve_str == "P-256" {
        curve = Curve::gen_p256();
    } 
    else if curve_str == "P-384" {
        curve = Curve::gen_p384();
    } 
    else if curve_str == "P-521" {
        curve = Curve::gen_p521();
    } 
    else {
        eprintln!("Valid curves are P-256, P-384, P-521.");
        return;
    }

    let d : Int;
    if backdoor_str == "" {
        d = seed_rng.gen_uint(curve.bitsize); 
    } 
    else {
        d = Int::from_str_radix(&backdoor_str, 10).unwrap();
        if d < 2 {
            eprintln!("Backdoor must be greater than 2.");
            return;
        }
    }
    

    let window = pancurses::initscr();

    let seed = seed_rng.gen_uint(curve.bitsize); 
    let q = curve.multiply(&curve.g, &d.mod_invert(&curve.n).unwrap());
    let mut prng = DualECDRBG::new(&curve, &seed, &curve.g, &q);

    window.printw(format!("Curve = \t{}\n", curve.name));
    window.printw(format!("Seed = \t\t{}\n", seed.to_str_radix(16, false)));
    window.printw(format!("d = \t\t{}\n", d.to_str_radix(16, false)));
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

    let begy = cy + 7; 
    let nlines = my - begy;
    window.mv(begy - 1, cx);
    window.hline('-', 10000);
    window.mv(cy, cx);
    window.refresh();

    let subwindow = window.derwin(nlines, mx, begy, cx).unwrap();
    subwindow.setscrreg(0, nlines);
    subwindow.scrollok(true);

    let timestamp = time::precise_time_s();
    let prediction = predict(&prng, &d, &output1, &output2, &subwindow);
    window.printw(format!("Eve spent {} minutes calculating Alice's state.\n", (time::precise_time_s() - timestamp) / 60.0));

    match prediction {
        Some(state) => {
            window.printw(format!("Eve guessed Alice's state as {}.\n", &state.to_str_radix(16, false)));
        },
        None => {
            window.printw(format!("Eve was not able to guess Alice's state.\n")) ;
        }
    } 

    prng.print_state(&"Alice's actual state is ", &".\n", Some(&window));
    window.printw("\nPress any key to exit.\n");
    window.getch();

    pancurses::endwin();
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;
    use ramp::int::Int; 
    use ramp::RandomInt;
    use math::ModExtensions;
    use curves::Curve;

    #[test]
    fn test_positive_mod_invert() {
        let inverse = &Int::from(4).mod_invert(&Int::from(7));
        assert_eq!(inverse.unwrap(), Int::from(2));
    }

    #[test]
    fn test_negative_mod_invert() {
        let inverse = &Int::from(-4).mod_invert(&Int::from(7));
        assert_eq!(inverse.unwrap(), Int::from(5));
    }

    #[test]
    fn test_point_multiplication() {
        let curve = Curve::gen_p256();
        let mut rng = thread_rng();
        for _ in 0..10 {
            let p = curve.multiply(&curve.g, &rng.gen_uint(curve.bitsize));
            assert!(curve.is_on_curve(&p), format!("{} is not on the {} curve.", p, curve.name));
        }
    }
}

