use ramp::int::Int;

lazy_static! {
    static ref ONE : Int = Int::one();
    static ref TWO : Int = Int::from(2);
    static ref TWO_POW_2 : Int = TWO.pow(2); 
    static ref TWO_POW_4 : Int = TWO.pow(4); 
    static ref TWO_POW_8 : Int = TWO.pow(8); 
    static ref TWO_POW_16 : Int = TWO.pow(16); 
    static ref TWO_POW_32 : Int = TWO.pow(32); 
    static ref TWO_POW_94 : Int = TWO.pow(94); 
    static ref TWO_POW_96 : Int = TWO.pow(96); 
    static ref P256_P : Int = Int::from_str_radix("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff", 16).unwrap();
}

pub trait ModExtensions {
    fn modulo(&self, n : &Int) -> Int;
    fn mod_invert(&self, n : &Int) -> Option<Int>;
    fn mod_sqrt(&self, n : &Int) -> Option<Int>;
    fn p256_mod_sqrt(&self) -> Option<Int>;
}

impl ModExtensions for Int {

    fn modulo(&self, n : &Int) -> Int {
        let mut r = self.pow_mod(&ONE, n);
        if r < 0 {
            r += n;
        }
        r
    }

    fn mod_invert(&self, n : &Int) -> Option<Int> {
        let b = if self < &0 {
            n + self 
        } else {
            self.clone()
        };

        let mut t = Int::zero();
        let mut newt = Int::one();
        let mut r = n.clone();
        let mut newr = b;

        let mut tmp : Int;

        while newr != 0 {
            let quot = &r / &newr; 

            tmp = t;
            t = newt.clone();
            newt = tmp - &quot * &newt;

            tmp = r;
            r = newr.clone();
            newr = tmp - &quot * &newr;
        }

        if r > 1 {
            return None;
        }

        if t < 0 {
            t += n;
        }

        Some(t)
    }

    fn mod_sqrt(&self, p : &Int) -> Option<Int> {
        // Big number implementation of the Tonelli-Shanks algorithm 
        if self.pow_mod(&((p - 1) >> (1 as usize)), &p) != 1 {
            return None;
        }

        let mut q = p - 1;
        let mut ss = Int::zero(); 

        while (&q & 1) == 0 {
            ss += 1; 
            q >>= 1;
        }

        if ss == 1 {
            let r1 = self.pow_mod(&((p + 1) >> (2 as usize)), &p);
            return Some(r1);
        }

        let mut z = Int::from(2);
        while &z.pow_mod(&((p - 1) >> (1 as usize)), &p) == &(p - 1) {
            z += 1; 
        }
        let mut c = z.pow_mod(&q, p);
        let mut r = self.pow_mod(&((&q + 1) >> (1 as usize)), p);
        let mut t = self.pow_mod(&q, p);
        let mut m = ss;

        loop {
            if t == 1 {
                return Some(r);
            }
            let mut i = Int::zero(); 
            let mut zz = t.clone();
            while zz != 1 && i < (&m - 1) {
                zz = (&zz * &zz).modulo(p); 
                i += 1;
            }
            let mut b = c.clone();
            let mut e = &m - &i - 1;
            while e > 0 {
                b = (&b * &b).modulo(p);
                e -= 1;
            }

            r = (&r * &b).modulo(p);
            c = (&b * &b).modulo(p);
            t = (&t * &c).modulo(p);
            m = i.clone();
        }    
    }

    fn p256_mod_sqrt(&self) -> Option<Int> {
        // Fast version of mod_sqrt, only works for the prime modulus in the P-256 NIST curve
        let mut t1 = self.pow_mod(&TWO, &P256_P);
        t1 = (&t1 * self).modulo(&P256_P);
        let mut t2 = t1.pow_mod(&TWO_POW_2, &P256_P);
        t2 = (&t2 * &t1).modulo(&P256_P);
        let mut t3 = t2.pow_mod(&TWO_POW_4, &P256_P);
        t3 = (&t3 * &t2).modulo(&P256_P);
        let mut t4 = t3.pow_mod(&TWO_POW_8, &P256_P);
        t4 = (&t4 * &t3).modulo(&P256_P);
        let mut r = t4.pow_mod(&TWO_POW_16, &P256_P);
        r = (&r * &t4).modulo(&P256_P);
        r = r.pow_mod(&TWO_POW_32, &P256_P);
        r = (&r * self).modulo(&P256_P);
        r = r.pow_mod(&TWO_POW_96, &P256_P);
        r = (&r * self).modulo(&P256_P);
        r = r.pow_mod(&TWO_POW_94, &P256_P);
        if self == &r.pow_mod(&TWO, &P256_P) {
            return Some(r);
        }
        None 
    }
}

