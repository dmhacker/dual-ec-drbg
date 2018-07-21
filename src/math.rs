use num::traits::{Zero, One};
use num::bigint::BigInt;

pub fn mod_inverse(a: &BigInt, m : &BigInt) -> Option<BigInt> {
    let mut t : BigInt = Zero::zero();
    let mut tnew : BigInt = One::one();
    let mut r = m.clone();
    let mut rnew = a.clone();
    
    while !rnew.is_zero() {
        let told = t.clone();
        t = tnew.clone();
        tnew = &told - (&r / &rnew) * &tnew;

        let rold = r.clone();
        r = rnew.clone();
        rnew = &rold % &rnew;
    }

    if r > One::one() {
        return None;
    }

    if t < Zero::zero() {
        t = t + m;
    }

    Some(t)
}
