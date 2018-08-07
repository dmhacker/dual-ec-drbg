use std::rc::Rc;
use rug::{Integer, Assign};
use curves::Curve;
use points::{Point, CurvePoint};

pub struct DualECDRBG {
    pub curve : Curve,
    pub outsize : u32, 
    pub outmask : Integer,
    pub p : Point,
    pub q : Point,
    state : Integer
}

impl DualECDRBG {
    pub fn new(curve : &Curve, p: &Point, q: &Point, seed : &Integer) -> DualECDRBG {
        assert!(p.is_on_curve(&curve), "P must be on the curve.");
        assert!(q.is_on_curve(&curve), "Q must be on the curve.");

        // The first 16 bits are removed from every output
        let outsize = curve.bitsize - 16;

        // The AND bitmask is equivalent to 2^{bitsize} - 1
        // This produces a string of 1's that is `bitsize` in length
        let mut outmask = Integer::from(Integer::u_pow_u(2, outsize));
        outmask -= 1;

        DualECDRBG {
            curve: curve.clone(), 
            outsize: outsize,
            outmask: outmask, 
            p: p.clone(),
            q: q.clone(), 
            state: seed.clone() 
        }
    }

    pub fn next(&mut self) -> Integer {
        // Create a reference to a clone DRBG's curve parameters
        let curve = Rc::new(self.curve.clone());
        
        // Multiply P by the state s to get the new point sP
        let mut sp = CurvePoint::from(&self.p, &curve);
        sp *= &self.state;

        // Set the state to sP.x 
        self.state = sp.x;

        // Multiply Q by the new state t to get tQ = (sP.x)Q
        let mut tq = CurvePoint::from(&self.q, &curve);
        tq *= &self.state;

        // Truncate the first 16 bits off of tQ by applying a bitmask 
        // Return this as 'random' output
        tq.x & &self.outmask
    }

    pub fn next_bits(&mut self, bits : u32) -> Integer {
        let mut remaining = bits;
        let mut result = Integer::new();
        let mut buffer = Integer::new();
        while remaining > 0 {
            // Determine the maximum bits we can extract from one call to next()
            let amount = if remaining < self.outsize {
                remaining
            } 
            else {
                self.outsize
            };

            // Only preserve the significant bits we need 
            buffer.assign(self.next() >> (self.outsize - amount));

            // Apply the buffer to the result
            result <<= amount;
            result |= &buffer;

            // Decrease the bits remaining
            remaining -= amount;
        }
        return result;
    }

    pub fn print_state(&self, prefix : &str, suffix : &str) {
        println!("{}{}{}", prefix, self.state.to_string_radix(16), suffix);
    }
}
