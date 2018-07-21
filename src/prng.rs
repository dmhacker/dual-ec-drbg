use num::bigint::BigInt;
use num::traits::Zero;
use curves::Curve;
use points::CurvePoint;

pub struct DualECDRBG {
    pub curve : Curve,
    pub s : BigInt,
    pub p : CurvePoint,
    pub q : CurvePoint,
    pub prefix : BigInt
}

impl DualECDRBG {
    pub fn new(curve : &Curve, seed : &BigInt, p: &CurvePoint, q: &CurvePoint) -> DualECDRBG {
        assert!(curve.is_on_curve(p), "P must be on the curve");
        assert!(curve.is_on_curve(q), "Q must be on the curve");

        DualECDRBG {
            curve: curve.clone(),
            s: seed.clone(), 
            p: p.clone(),
            q: q.clone(),
            prefix: BigInt::from(0)
        }
    }

    pub fn next(&mut self) -> BigInt {
        let sp = self.curve.multiply(&self.p, &self.s);
        let r = sp.x;

        let rp = self.curve.multiply(&self.p, &r);
        self.s = rp.x;

        let rq = self.curve.multiply(&self.q, &r);
        let rqx = rq.x;

        let b = rqx.bits();
        let mut k : BigInt = Zero::zero();
        let mut i = 0;

        while i < b - 16 {
            k <<= 1;
            k += 1;
            i += 1;
        }

        self.prefix = &rqx >> (b - 16);

        rqx & k
    }
}
