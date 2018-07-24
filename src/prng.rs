use ramp::int::Int;
use curves::Curve;
use points::CurvePoint;
use pancurses::Window;

pub struct DualECDRBG {
    pub curve : Curve,
    pub outsize : usize, 
    pub outmask : Int,
    pub p : CurvePoint,
    pub q : CurvePoint,
    state : Int
}

impl DualECDRBG {
    pub fn new(curve : &Curve, seed : &Int, p: &CurvePoint, q: &CurvePoint) -> DualECDRBG {
        assert!(curve.is_on_curve(p), "P must be on the curve");
        assert!(curve.is_on_curve(q), "Q must be on the curve");

        let outsize = curve.bitsize - 16;

        DualECDRBG {
            curve: curve.clone(),
            outsize: outsize,
            outmask: Int::from(2).pow(outsize) - 1,
            p: p.clone(),
            q: q.clone(),
            state: seed.clone() 
        }
    }

    pub fn next(&mut self) -> Int {
        let sp = self.curve.multiply(&self.p, &self.state);
        let s = sp.x.clone();

        let s1q = self.curve.multiply(&self.q, &s);
        let r = s1q.x.clone();

        self.state = s;

        r & &self.outmask
    }

    pub fn print_state(&self, prefix : &str, suffix : &str, window : Option<&Window>) {
        match window {
            Some(window) => {
                window.printw(format!("{}{}{}", prefix, self.state.to_str_radix(16, false), suffix));
            },
            None => {
                println!("{}{}{}", prefix, self.state.to_str_radix(16, false), suffix);
            }
        };
    }
}
