use num::bigint::BigInt;
use num::traits::Zero;
use curves::Curve;
use points::CurvePoint;

pub struct DualECDRBG {
    pub curve : Curve,
    pub s : BigInt,
    pub p : CurvePoint,
    pub q : CurvePoint,
    pub bitmask : BigInt
}

impl DualECDRBG {
    pub fn new(curve : &Curve, seed : &BigInt, p: &CurvePoint, q: &CurvePoint) -> DualECDRBG {
        assert!(curve.is_on_curve(p), "P must be on the curve");
        assert!(curve.is_on_curve(q), "Q must be on the curve");

        let bitsize = curve.bitsize - 16;
        let mut bitmask : BigInt = Zero::zero();

        for _ in 0..bitsize {
            bitmask <<= 1;
            bitmask += 1;
        }

        DualECDRBG {
            curve: curve.clone(),
            s: seed.clone(), 
            p: p.clone(),
            q: q.clone(),
            bitmask: bitmask,
        }
    }

    pub fn next(&mut self) -> BigInt {
        let sp = self.curve.multiply(&self.p, &self.s);
        let s = sp.x.clone();

        let s1q = self.curve.multiply(&self.q, &s);
        let r = s1q.x.clone();

        self.s = s.clone();

        r & &self.bitmask
    }
}
