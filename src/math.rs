use rand::Rng;
use rug::{Assign, Integer};

pub trait ModuloExt {
    fn modulo(&self, n: &Integer) -> Integer;
    fn modulo_mut(&mut self, n: &Integer);
    fn sqrt_mod(&self, n: &Integer) -> Option<Integer>;
}

lazy_static! {
    static ref ONE: Integer = Integer::from(1);
    static ref TWO: Integer = Integer::from(2);
}

impl ModuloExt for Integer {
    // Computes a new integer r, such that r = self mod n
    fn modulo(&self, n: &Integer) -> Integer {
        let mut r = self.clone();
        r.pow_mod_mut(&ONE, n).unwrap();
        r
    }

    // Computes self = self mod n
    fn modulo_mut(&mut self, n: &Integer) {
        self.pow_mod_mut(&ONE, n).unwrap();
    }

    // Computes a new integer r, such that r^2 = self (mod p)
    // The modulus p must be a prime number for this to work
    fn sqrt_mod(&self, p: &Integer) -> Option<Integer> {
        // Big number implementation of the Tonelli-Shanks algorithm
        let mut tmp = Integer::from(p - 1);
        tmp >>= 1;
        let mut buffer = self.clone();
        buffer.pow_mod_mut(&tmp, &p).unwrap();
        if buffer != 1 {
            return None;
        }

        let mut q = Integer::from(p - 1);
        let mut ss = Integer::from(0);

        tmp.assign(&q & 1);
        while tmp == 0 {
            ss += 1;
            q >>= 1;
            tmp.assign(&q & 1);
        }

        if ss == 1 {
            tmp.assign(p + 1);
            tmp >>= 2;
            buffer.assign(self);
            buffer.pow_mod_mut(&tmp, &p).unwrap();
            return Some(buffer);
        }

        let mut z = Integer::from(2);
        tmp.assign(p - 1);
        tmp >>= 1;
        buffer.assign(p - 1);
        while z.clone().pow_mod(&tmp, &p).unwrap() == buffer {
            z += 1;
        }
        let mut c = z.pow_mod(&q, p).unwrap();
        tmp.assign(&q + 1);
        tmp >>= 1;
        let mut r = self.clone();
        r.pow_mod_mut(&tmp, p).unwrap();
        let mut t = self.clone();
        r.pow_mod_mut(&q, p).unwrap();
        let mut m = ss;

        loop {
            if t == 1 {
                return Some(r);
            }
            let mut i = Integer::new();
            let mut zz = t.clone();
            tmp.assign(&m - 1);
            while zz != 1 && i < tmp {
                zz.pow_mod_mut(&TWO, p).unwrap();
                i += 1;
            }
            let mut b = c.clone();
            let mut e = Integer::from(&m - &i);
            e -= 1;
            while e > 0 {
                b.pow_mod_mut(&TWO, p).unwrap();
                e -= 1;
            }

            r *= &b;
            r.modulo_mut(p);
            c.assign(&b * &b);
            c.modulo_mut(p);
            t *= &c;
            t.modulo_mut(p);
            m = i.clone();
        }
    }
}

pub trait RandExt {
    fn gen_uint(&mut self, bits: u32) -> Integer;
}

impl<R: Rng> RandExt for R {
    // Computes a random large integer that has `bits` number of bits
    fn gen_uint(&mut self, bits: u32) -> Integer {
        let mut bits_remaining = bits;
        let mut bitmask = Integer::new();
        let mut randint = Integer::new();

        while bits_remaining > 0 {
            // Generate 32 random bits
            let mut generated: u32 = self.gen();

            // Compute the exact amount of bits needed for the final result
            // If more bits are needed, set the max as 32
            let amount = if bits_remaining > 32 {
                32
            } else {
                bits_remaining
            };

            // The AND bitmask is equivalent to 2^{amount} - 1
            // This produces a string of `amount` 1's
            bitmask.assign(Integer::u_pow_u(2, amount as u32));
            bitmask -= 1;

            // Truncate the generated bits to the amount we need
            generated &= bitmask.to_u32().unwrap();

            // Shift the current bits over and add the newly generated ones
            randint <<= amount as u32;
            randint |= &generated;

            // Decrease the amount of bits that we need to completion
            bits_remaining -= amount;
        }

        randint
    }
}
