use num::traits::{Zero, One};
use num::bigint::BigInt;

pub fn prime_mod_inverse(a: &BigInt, n : &BigInt) -> BigInt {
    // Function only works when n (the modulus) is prime 
    a.modpow(&(n.clone() - 2), &n) 
}

pub fn mod_inverse(a0 : &BigInt, n0 : &BigInt) -> Option<BigInt> {
    // Function only works when a0 is positive currently
    let a = a0.clone();
    let mut n = n0.clone();
    let z : BigInt = Zero::zero();
    let o : BigInt = One::one();

    let mut t : BigInt = Zero::zero();
    let mut nt : BigInt = One::one();
    let mut r = n.clone();
    let mut nr = &a % &n;

    if &n < &z {
       n = &z - n;
    }

    while !nr.is_zero() {
        let quot = &r / &nr;

        let mut tmp = nt.clone();
        nt = &t - &quot * &nt;
        t = tmp.clone();

        tmp = nr.clone();
        nr = &r - &quot * &nr;
        r = tmp.clone();
    }

    if &r > &o {
        return None;
    }

    if &t < &z {
        t += &n;
    }

    Some(t)
}

pub fn tonelli_shanks(n : &BigInt, p : &BigInt) -> Option<BigInt> {
    let one : BigInt = One::one();
    let zero : BigInt = Zero::zero();
    let one_usize : usize = 1;
    let two_usize : usize = 2;

    if n.modpow(&((p - 1) >> one_usize), &p) != one {
        return None;
    }

    let mut q = p - &one;
    let mut ss = zero.clone();

    while (&q & &one) == zero {
        ss += &one;
        q >>= 1;
    }

    if ss == one {
        let r1 = n.modpow(&((p + 1) >> two_usize), &p);
        return Some(r1);
    }

    let mut z = BigInt::from(2);
    while &z.modpow(&((p - 1) >> one_usize), &p) == &(p - 1) {
        z += &one;
    }
    let mut c = z.modpow(&q, p);
    let mut r = n.modpow(&((&q + 1) >> one_usize), p);
    let mut t = n.modpow(&q, p);
    let mut m = ss.clone();

    loop {
        if t == one {
            return Some(r);
        }
        let mut i = zero.clone();
        let mut zz = t.clone();
        while zz != one && i < (&m - &one) {
            zz = (&zz * &zz).modpow(&one, p); 
            i += &one;
        }
        let mut b = c.clone();
        let mut e = &m - &i - 1;
        while e > zero {
            b = (&b * &b).modpow(&one, p);
            e -= &one;
        }

        r = (&r * &b).modpow(&one, p);
        c = (&b * &b).modpow(&one, p);
        t = (&t * &c).modpow(&one, p);
        m = i.clone();
    }
}
