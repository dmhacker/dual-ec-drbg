use ramp::int::Int;

pub fn modulo(a : &Int, n : &Int) -> Int {
    let mut r = a.pow_mod(&Int::one(), n);
    if r < 0 {
        r += n;
    }
    r
}

pub fn mod_inverse(a : &Int, n: &Int) -> Option<Int> {
    let mut u1 = Int::one();
    let mut u3 = (*a).clone();
    let mut v1 = Int::zero();
    let mut v3 = (*n).clone();

    let mut iter = true;

    while v3 != Int::zero()
    {
        let q = &u3 / &v3;
        let t3 = u3 % &v3;
        let t1 = u1 + &q * &v1;

        u1 = v1.clone();
        v1 = t1.clone();
        u3 = v3.clone();
        v3 = t3.clone();

        iter = !iter;
    }

    if u3 != Int::one() {
        return None;
    }

    let inv = if iter == false {
        n - u1
    } else {
        u1
    };

    Some(inv)
}

pub fn prime_mod_inverse(a: &Int, n : &Int) -> Option<Int> {
    // Function only works when n (the modulus) is prime 
    let mut r = a.pow_mod(&(n - 2), &n);
    if r < 0 {
        r += n;
    }
    Some(r)
}

pub fn mod_sqrt(n : &Int, p : &Int) -> Option<Int> {
    // Big number implementation of the Tonelli-Shanks algorithm 
    if n.pow_mod(&((p - 1) >> (1 as usize)), &p) != 1 {
        return None;
    }

    let mut q = p - 1;
    let mut ss = Int::zero(); 

    while (&q & 1) == 0 {
        ss += 1; 
        q >>= 1;
    }

    if ss == 1 {
        let r1 = n.pow_mod(&((p + 1) >> (2 as usize)), &p);
        return Some(r1);
    }

    let mut z = Int::from(2);
    while &z.pow_mod(&((p - 1) >> (1 as usize)), &p) == &(p - 1) {
        z += 1; 
    }
    let mut c = z.pow_mod(&q, p);
    let mut r = n.pow_mod(&((&q + 1) >> (1 as usize)), p);
    let mut t = n.pow_mod(&q, p);
    let mut m = ss.clone();

    loop {
        if t == 1 {
            return Some(r);
        }
        let mut i = Int::zero(); 
        let mut zz = t.clone();
        while zz != 1 && i < (&m - 1) {
            zz = modulo(&(&zz * &zz), p); 
            i += 1;
        }
        let mut b = c.clone();
        let mut e = &m - &i - 1;
        while e > 0 {
            b = modulo(&(&b * &b), p);
            e -= 1;
        }

        r = modulo(&(&r * &b), p);
        c = modulo(&(&b * &b), p);
        t = modulo(&(&t * &c), p);
        m = i.clone();
    }
}

pub fn p256_mod_sqrt(c : &Int) -> Option<Int> {
    // Fast version of mod_sqrt, only works for the prime modulus in the P-256 NIST curve
    let two = Int::from(2); 
    let p = Int::from_str_radix("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff", 16).unwrap();
    let mut t1 = c.pow_mod(&two, &p);
    t1 = modulo(&(&t1 * c), &p);
    let mut t2 = t1.pow_mod(&two.pow(2), &p);
    t2 = modulo(&(&t2 * &t1), &p);
    let mut t3 = t2.pow_mod(&two.pow(4), &p);
    t3 = modulo(&(&t3 * &t2), &p);
    let mut t4 = t3.pow_mod(&two.pow(8), &p);
    t4 = modulo(&(&t4 * &t3), &p);
    let mut r = t4.pow_mod(&two.pow(16), &p);
    r = modulo(&(&r * &t4), &p);
    r = r.pow_mod(&two.pow(32), &p);
    r = modulo(&(&r * c), &p);
    r = r.pow_mod(&two.pow(96), &p);
    r = modulo(&(&r * c), &p);
    r = r.pow_mod(&two.pow(94), &p);
    if c == &r.pow_mod(&two, &p) {
        return Some(r);
    }
    None 
}
