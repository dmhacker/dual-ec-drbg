use num::traits::{Zero, One};
use num::bigint::BigInt;

pub fn modulo(a : &BigInt, m : &BigInt) -> BigInt {
    a.modpow(&One::one(), m)
}

pub fn mod_inverse(a : &BigInt, n : &BigInt) -> Option<BigInt> {
    // Function only works when a is positive currently
    let z : BigInt = Zero::zero();
    let o : BigInt = One::one();

    assert!(n > &z, "Modulus must be positive");

    let mut t : BigInt = Zero::zero();
    let mut nt : BigInt = One::one();
    let mut r = n.clone();
    let mut nr = a % n;

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
        t += n;
    }

    Some(t)
}

pub fn prime_mod_inverse(a: &BigInt, n : &BigInt) -> Option<BigInt> {
    // Function only works when n (the modulus) is prime 
    Some(a.modpow(&(n.clone() - 2), &n))
}

pub fn mod_sqrt(n : &BigInt, p : &BigInt) -> Option<BigInt> {
    // Big number implementation of the Tonelli-Shanks algorithm 
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
        ss += 1; 
        q >>= 1;
    }

    if ss == one {
        let r1 = n.modpow(&((p + 1) >> two_usize), &p);
        return Some(r1);
    }

    let mut z = BigInt::from(2);
    while &z.modpow(&((p - 1) >> one_usize), &p) == &(p - 1) {
        z += 1; 
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
            zz = modulo(&(&zz * &zz), p); 
            i += 1;
        }
        let mut b = c.clone();
        let mut e = &m - &i - 1;
        while e > zero {
            b = modulo(&(&b * &b), p);
            e -= 1;
        }

        r = modulo(&(&r * &b), p);
        c = modulo(&(&b * &b), p);
        t = modulo(&(&t * &c), p);
        m = i.clone();
    }
}

pub fn p256_mod_sqrt(c : &BigInt, p : &BigInt) -> Option<BigInt> {
    // Fast version of mod_sqrt, only works for the prime modulus in the P-256 NIST curve
    let two = BigInt::from(2); 
    let f = BigInt::parse_bytes(b"ffffffff00000001000000000000000000000000ffffffffffffffffffffffff", 16).unwrap();
    let mut t1 = c.modpow(&two, &f);
    t1 = modulo(&(&t1 * c), &f);
    let mut t2 = t1.modpow(&two_pow(2), &f);
    t2 = modulo(&(&t2 * &t1), &f);
    let mut t3 = t2.modpow(&two_pow(4), &f);
    t3 = modulo(&(&t3 * &t2), &f);
    let mut t4 = t3.modpow(&two_pow(8), &f);
    t4 = modulo(&(&t4 * &t3), &f);
    let mut r = t4.modpow(&two_pow(16), &f);
    r = modulo(&(&r * &t4), &f);
    r = r.modpow(&two_pow(32), &f);
    r = modulo(&(&r * c), &f);
    r = r.modpow(&two_pow(96), &f);
    r = modulo(&(&r * c), &f);
    r = r.modpow(&two_pow(94), &f);
    if c == &r.modpow(&two, &p) {
        return Some(r);
    }
    None 
}

pub fn two_pow(n : usize) -> BigInt {
    // Computes 2 ** n
    let mut r : BigInt = Zero::zero(); 
    for _ in 0..n {
        r <<= 1;
        r += 1;
    }
    r + 1
}
