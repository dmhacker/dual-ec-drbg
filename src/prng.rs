use std::rc::Rc;
use rug::Integer;
use curves::Curve;
use points::{Point, CurvePoint};
use pancurses::Window;

pub struct DualECDRBG {
    pub curve : Curve,
    pub outsize : u32, 
    pub outmask : Integer,
    pub p : Point,
    pub q : Point,
    state : Integer
}

impl DualECDRBG {
    pub fn new(curve : &Curve, seed : &Integer, p: &CurvePoint, q: &CurvePoint) -> DualECDRBG {
        assert!(curve.is_on_curve(p), "P must be on the curve");
        assert!(curve.is_on_curve(q), "Q must be on the curve");

        let outsize = curve.bitsize - 16;

        let mut outmask = Integer::from(Integer::u_pow_u(2, outsize));
        outmask -= 1;

        DualECDRBG {
            curve: curve.clone(),
            outsize: outsize,
            outmask: outmask, 
            p: Point::from(p),
            q: Point::from(q),
            state: seed.clone() 
        }
    }

    pub fn next(&mut self) -> Integer {
        let curve = Rc::new(self.curve.clone());

        let sp = &self.p.convert(Rc::clone(&curve)) * &self.state;
        let s = sp.x.clone();

        let s1q = &self.q.convert(Rc::clone(&curve)) * &s;
        let r = s1q.x.clone();

        self.state = s;

        r & &self.outmask
    }

    pub fn print_state(&self, prefix : &str, suffix : &str, window : Option<&Window>) {
        match window {
            Some(window) => {
                window.printw(format!("{}{}{}", prefix, self.state.to_string_radix(16), suffix));
            },
            None => {
                println!("{}{}{}", prefix, self.state.to_string_radix(16), suffix);
            }
        };
    }
}
