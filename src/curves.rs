use num::bigint::BigInt;
use num::traits::One;
use points::CurvePoint;
use math::mod_inverse;

#[derive(Debug)]
pub struct Curve {
    pub name: String,   // Name of the curve
    pub bitsize: u16,   // Level of security offered by the curve 
    pub p: BigInt,      // Order of the finite field that the curve belongs to (mod p)
    pub n: BigInt,      // Order of the elliptic curve group
    pub a: BigInt,      // a parameter in the curve equation
    pub b: BigInt,      // b parameter in the curve equation
    pub g: CurvePoint   // Generator (base) point
}

impl Curve {
    fn _lambda(&self, p : &CurvePoint, q : &CurvePoint, numer : &BigInt, denom : &BigInt) -> CurvePoint {
        let one : BigInt = One::one();

        let lambda = (numer * mod_inverse(denom, &self.p).unwrap()).modpow(&one, &self.p);
        let rx = (&lambda * &lambda - &p.x - &q.x).modpow(&one, &self.p);
        let ry = (lambda * (&p.x - &rx) - &p.y).modpow(&one, &self.p);

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

    pub fn multiply(&self, p : &CurvePoint, s : &BigInt) -> CurvePoint {
        let mut q = p.clone(); 
        let mut q0 = true;

        let one : BigInt = One::one();

        let m = s.bits();
        let mut i = m - 1;

        loop {
            q = self._double(&q);

            let di = (s >> i) & &one;
            if di == one {
                if q0 {
                    q = p.clone();
                    q0 = false;
                }
                else {
                    q = self.add(&q, &p);
                }
            }

            if i == 0 {
                break;
            }
            else {
                i -= 1;
            }
        }

        q 
    }

    pub fn is_on_curve(&self, p : &CurvePoint) -> bool {
        let lhs = (&p.y * &p.y) % &self.p;
        let rhs = ((&p.x * &p.x * &p.x) + (&self.a * &p.x) + &self.b) % &self.p;
        (lhs == rhs)
    }

    pub fn gen_p256() -> Curve {
        let generator = CurvePoint {
            x: BigInt::parse_bytes(b"6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296", 16).unwrap(),
            y: BigInt::parse_bytes(b"4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5", 16).unwrap()
        };
        Curve {
            name: String::from("P-256"),
            bitsize: 256,
            p: BigInt::parse_bytes(b"115792089210356248762697446949407573530086143415290314195533631308867097853951", 10).unwrap(),
            n: BigInt::parse_bytes(b"115792089210356248762697446949407573529996955224135760342422259061068512044369", 10).unwrap(),
            a: BigInt::parse_bytes(b"-3", 10).unwrap(),
            b: BigInt::parse_bytes(b"5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b", 16).unwrap(),
            g: generator 
        }
    }

    pub fn gen_p384() -> Curve {
        let generator = CurvePoint {
            x: BigInt::parse_bytes(b"aa87ca22be8b05378eb1c71ef320ad746e1d3b628ba79b9859f741e082542a385502f25dbf55296c3a545e3872760ab7", 16).unwrap(),
            y: BigInt::parse_bytes(b"3617de4a96262c6f5d9e98bf9292dc29f8f41dbd289a147ce9da3113b5f0b8c00a60b1ce1d7e819d7a431d7c90ea0e5f", 16).unwrap()
        };
        Curve {
            name: String::from("P-384"),
            bitsize: 384,
            p: BigInt::parse_bytes(b"39402006196394479212279040100143613805079739270465446667948293404245721771496870329047266088258938001861606973112319", 10).unwrap(),
            n: BigInt::parse_bytes(b"39402006196394479212279040100143613805079739270465446667946905279627659399113263569398956308152294913554433653942643", 10).unwrap(),
            a: BigInt::parse_bytes(b"-3", 10).unwrap(),
            b: BigInt::parse_bytes(b"b3312fa7e23ee7e4988e056be3f82d19181d9c6efe8141120314088f5013875ac656398d8a2ed19d2a85c8edd3ec2aef", 16).unwrap(),
            g: generator
        }
    }

    pub fn gen_p521() -> Curve {
        let generator = CurvePoint {
            x: BigInt::parse_bytes(b"c6858e06b70404e9cd9e3ecb662395b4429c648139053fb521f828af606b4d3dbaa14b5e77efe75928fe1dc127a2ffa8de3348b3c1856a429bf97e7e31c2e5bd66", 16).unwrap(),
            y: BigInt::parse_bytes(b"11839296a789a3bc0045c8a5fb42c7d1bd998f54449579b446817afbd17273e662c97ee72995ef42640c550b9013fad0761353c7086a272c24088be94769fd16650", 16).unwrap()
        };
        Curve {
            name: String::from("P-521"),
            bitsize: 521,
            p: BigInt::parse_bytes(b"6864797660130609714981900799081393217269435300143305409394463459185543183397656052122559640661454554977296311391480858037121987999716643812574028291115057151", 10).unwrap(),
            n: BigInt::parse_bytes(b"6864797660130609714981900799081393217269435300143305409394463459185543183397655394245057746333217197532963996371363321113864768612440380340372808892707005449", 10).unwrap(),
            a: BigInt::parse_bytes(b"-3", 10).unwrap(),
            b: BigInt::parse_bytes(b"051953eb9618e1c9a1f929a21a0b68540eea2da725b99b315f3b8b489918ef109e156193951ec7e937b1652c0bd3bb1bf073573df883d2c34f1ef451fd46b503f00", 16).unwrap(),
            g: generator,
        }
    }
}
