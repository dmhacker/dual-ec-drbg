use ramp::int::Int;
use curves::Curve;
use points::CurvePoint;

pub struct DualECDRBG {
    pub curve : Curve,
    pub p : CurvePoint,
    pub q : CurvePoint,
    pub s : Int,
    bitmask : Int 
}

impl DualECDRBG {
    pub fn new(curve : &Curve, seed : &Int, p: &CurvePoint, q: &CurvePoint) -> DualECDRBG {
        assert!(curve.is_on_curve(p), "P must be on the curve");
        assert!(curve.is_on_curve(q), "Q must be on the curve");

        DualECDRBG {
            curve: curve.clone(),
            s: seed.clone(), 
            p: p.clone(),
            q: q.clone(),
            bitmask: Int::from(2).pow(curve.bitsize - 16) - 1
        }
    }

    pub fn next(&mut self) -> Int {
        let sp = self.curve.multiply(&self.p, &self.s);
        let s = sp.x.clone();

        let s1q = self.curve.multiply(&self.q, &s);
        let r = s1q.x.clone();

        self.s = s.clone();

        r & &self.bitmask
    }
}
