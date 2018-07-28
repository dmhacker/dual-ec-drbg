use rug::{Integer, Assign};
use rand::Rng;

lazy_static! {
    static ref ONE : Integer = Integer::from(1);
    static ref TWO : Integer = Integer::from(2);
    static ref TWO_POW_2 : Integer = Integer::from(Integer::u_pow_u(2, 2)); 
    static ref TWO_POW_4 : Integer = Integer::from(Integer::u_pow_u(2, 4)); 
    static ref TWO_POW_8 : Integer = Integer::from(Integer::u_pow_u(2, 8)); 
    static ref TWO_POW_16 : Integer = Integer::from(Integer::u_pow_u(2, 16)); 
    static ref TWO_POW_32 : Integer = Integer::from(Integer::u_pow_u(2, 32)); 
    static ref TWO_POW_94 : Integer = Integer::from(Integer::u_pow_u(2, 94)); 
    static ref TWO_POW_96 : Integer = Integer::from(Integer::u_pow_u(2, 96)); 
    static ref P256_P : Integer = Integer::from_str_radix("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff", 16).unwrap();
}

pub trait ModExtensions {
    fn modulo(&self, n : &Integer) -> Integer;
    fn modulo_mut(&mut self, n : &Integer);
    fn sqrt_mod(&self, n : &Integer) -> Option<Integer>;
}

impl ModExtensions for Integer {

    fn modulo(&self, n : &Integer) -> Integer {
        let mut r = self.clone();
        r.pow_mod_mut(&ONE, n).unwrap();
        r
    }

    fn modulo_mut(&mut self, n : &Integer) {
        self.pow_mod_mut(&ONE, n).unwrap();
    }

    fn sqrt_mod(&self, p : &Integer) -> Option<Integer> {
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

pub trait RandExtensions {
    fn gen_uint(&mut self, bits : u32) -> Integer;
}

impl<R: Rng> RandExtensions for R {
    fn gen_uint(&mut self, bits : u32) -> Integer {
        let mut bits_remaining = bits;
        let mut bitmask = Integer::new();
        let mut randint = Integer::new();
        let mut buffer = Integer::new();
        while bits_remaining > 0 {
            let generated : u32 = self.gen();
            let amount = if bits_remaining > 32 {
                32
            }
            else {
                bits_remaining
            };
            bitmask.assign(Integer::u_pow_u(2, amount as u32));
            bitmask -= 1;
            buffer.assign(generated & &bitmask);
            randint <<= amount as u32;
            randint |= &buffer;
            bits_remaining -= amount;
        }
        randint
    }
}
