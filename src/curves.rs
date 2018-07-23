use ramp::int::Int;
use points::CurvePoint;
use math::{mod_inverse, prime_mod_inverse, modulo};

#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    pub name: String,       // Name of the curve
    pub bitsize: usize,     // Level of security offered by the curve 
    pub p: Int,             // Order of the finite field that the curve belongs to (mod p)
    pub n: Int,             // Order of the elliptic curve group
    pub a: Int,             // a parameter in the curve equation
    pub b: Int,             // b parameter in the curve equation
    pub g: CurvePoint       // Generator (base) point
}

impl Curve {
    #[inline]
    fn _lambda(&self, p : &CurvePoint, q : &CurvePoint, numer : &Int, denom : &Int) -> CurvePoint {
        let denom_inverse : Int;
        if &self.name[..2] == "P-" {
            denom_inverse = prime_mod_inverse(denom, &self.p).unwrap();
        }
        else {
            denom_inverse = mod_inverse(denom, &self.p).unwrap();
        }

        let lambda = modulo(&(numer * &denom_inverse), &self.p);
        let rx = modulo(&(&lambda * &lambda - &p.x - &q.x), &self.p);
        let ry = modulo(&(&lambda * (&p.x - &rx) - &p.y), &self.p);

        CurvePoint {
            x: rx,
            y: ry 
        }
    }

    fn _double(&self, p : &CurvePoint) -> CurvePoint {
        let numer = 3 * &p.x * &p.x + &self.a;
        let denom = 2 * &p.y;
        self._lambda(p, p, &numer, &denom) 
    }

    pub fn add(&self, p : &CurvePoint, q : &CurvePoint) -> CurvePoint {
        if p == q {
            return self._double(p);
        }

        let numer = &q.y - &p.y;
        let denom = &q.x - &p.x;
        self._lambda(p, q, &numer, &denom) 
    }

    pub fn multiply(&self, p : &CurvePoint, s : &Int) -> CurvePoint {
        let mut r0 = p.clone(); 
        let mut r1 = self._double(&p);

        let m = s.bit_length() as usize;
        let mut i = m - 2;

        loop {
            let di = (s >> i) & 1;
            if di == 1 {
                r0 = self.add(&r1, &r0);
                r1 = self._double(&r1);
            }
            else {
                r1 = self.add(&r0, &r1);
                r0 = self._double(&r0);
            }

            if i == 0 {
                break;
            }
            else {
                i -= 1;
            }
        }

        r0
    }

    pub fn is_on_curve(&self, p : &CurvePoint) -> bool {
        let lhs = (&p.y * &p.y) % &self.p;
        let rhs = ((&p.x * &p.x * &p.x) + (&self.a * &p.x) + &self.b) % &self.p;
        (lhs == rhs)
    }

    pub fn gen_p256() -> Curve {
        let generator = CurvePoint {
            x: Int::from_str_radix("6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296", 16).unwrap(),
            y: Int::from_str_radix("4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5", 16).unwrap()
        };
        Curve {
            name: String::from("P-256"),
            bitsize: 256,
            p: Int::from_str_radix("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff", 16).unwrap(),
            n: Int::from_str_radix("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551", 16).unwrap(),
            a: Int::from(-3), 
            b: Int::from_str_radix("5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b", 16).unwrap(),
            g: generator 
        }
    }

    pub fn gen_p384() -> Curve {
        let generator = CurvePoint {
            x: Int::from_str_radix("aa87ca22be8b05378eb1c71ef320ad746e1d3b628ba79b9859f741e082542a385502f25dbf55296c3a545e3872760ab7", 16).unwrap(),
            y: Int::from_str_radix("3617de4a96262c6f5d9e98bf9292dc29f8f41dbd289a147ce9da3113b5f0b8c00a60b1ce1d7e819d7a431d7c90ea0e5f", 16).unwrap()
        };
        Curve {
            name: String::from("P-384"),
            bitsize: 384,
            p: Int::from_str_radix("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000ffffffff", 16).unwrap(),
            n: Int::from_str_radix("ffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52973", 16).unwrap(),
            a: Int::from(-3), 
            b: Int::from_str_radix("b3312fa7e23ee7e4988e056be3f82d19181d9c6efe8141120314088f5013875ac656398d8a2ed19d2a85c8edd3ec2aef", 16).unwrap(),
            g: generator
        }
    }

    pub fn gen_p521() -> Curve {
        let generator = CurvePoint {
            x: Int::from_str_radix("c6858e06b70404e9cd9e3ecb662395b4429c648139053fb521f828af606b4d3dbaa14b5e77efe75928fe1dc127a2ffa8de3348b3c1856a429bf97e7e31c2e5bd66", 16).unwrap(),
            y: Int::from_str_radix("11839296a789a3bc0045c8a5fb42c7d1bd998f54449579b446817afbd17273e662c97ee72995ef42640c550b9013fad0761353c7086a272c24088be94769fd16650", 16).unwrap()
        };
        Curve {
            name: String::from("P-521"),
            bitsize: 521,
            p: Int::from_str_radix("20000000000006000000000000000000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", 16).unwrap(),
            n: Int::from_str_radix("20000000000006000000000000000000ffffffffffffffffffffffffffffffffffa51868783bf2f966b7fcc0148f709a5d03bb5c9b8899c47aebb6fb71e91386409", 16).unwrap(),
            a: Int::from(-3),
            b: Int::from_str_radix("051953eb9618e1c9a1f929a21a0b68540eea2da725b99b315f3b8b489918ef109e156193951ec7e937b1652c0bd3bb1bf073573df883d2c34f1ef451fd46b503f00", 16).unwrap(),
            g: generator,
        }
    }
}
