use crate::curves::Curve;
use crate::math::ModuloExt;
use rug::{Assign, Integer};
use std::fmt::{Display, Formatter, Result};
use std::ops::{Add, AddAssign, Mul, MulAssign};
use std::rc::Rc;

// A Point represents any 2-dimensional coordinate in Euclidean space
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub x: Integer,
    pub y: Integer,
}

// A CurvePoint is similar to a Point, only that is contains a reference to a curve that it is on
#[derive(Clone, Debug, PartialEq)]
pub struct CurvePoint {
    pub x: Integer,
    pub y: Integer,
    pub curve: Rc<Curve>,
}

impl Point {
    // Returns true if the point is on the curve, false otherwise
    pub fn is_on_curve(&self, curve: &Curve) -> bool {
        // Use lhs as a temporary buffer for computing a * x
        let mut lhs = Integer::from(&curve.a * &self.x);

        // Compute rhs = (x^3 + ax + b) mod p
        let mut rhs = Integer::from(&self.x * &self.x);
        rhs *= &self.x;
        rhs += &lhs;
        rhs += &curve.b;
        rhs.modulo_mut(&curve.p);

        // Compute lhs = (y^2) mod p
        lhs.assign(&self.y * &self.y);
        lhs.modulo_mut(&curve.p);

        // Check for equality
        lhs == rhs
    }
}

impl<'a> From<&'a CurvePoint> for Point {
    fn from(point: &'a CurvePoint) -> Point {
        Point {
            x: point.x.clone(),
            y: point.y.clone(),
        }
    }
}

impl CurvePoint {
    // Produces a point on the curve given a 2-D point and a reference to a curve
    pub fn from(point: &Point, curve: &Rc<Curve>) -> CurvePoint {
        assert!(
            point.is_on_curve(&curve),
            "The converted point must be on the curve."
        );

        CurvePoint {
            x: point.x.clone(),
            y: point.y.clone(),
            curve: curve.clone(),
        }
    }
}

// Intermediate step in elliptic curve point addition & doubling
#[inline]
fn _lambda(p: &CurvePoint, q: &CurvePoint, numer: Integer, mut denom: Integer) -> CurvePoint {
    denom.invert_mut(&p.curve.p).unwrap();

    // Lambda value is calculated using the given numerator and denominator
    let mut lambda = Integer::from(&numer * &denom);
    lambda.modulo_mut(&p.curve.p);

    // Lambda is then used to produce the x value of the new coordinate
    let mut rx = Integer::from(&lambda * &lambda);
    rx -= &p.x;
    rx -= &q.x;
    rx.modulo_mut(&p.curve.p);

    // Also used to produce the y value of the new coordinate
    let mut ry = Integer::from(&p.x - &rx);
    ry *= &lambda;
    ry -= &p.y;
    ry.modulo_mut(&p.curve.p);

    CurvePoint {
        x: rx,
        y: ry,
        curve: Rc::clone(&p.curve),
    }
}

// Produces a curve point R such that R = 2P = P + P
#[inline]
fn _double(p: &CurvePoint) -> CurvePoint {
    let mut numer = Integer::from(3);
    numer *= &p.x;
    numer *= &p.x;
    numer += &p.curve.a;

    let mut denom = Integer::from(2);
    denom *= &p.y;

    _lambda(p, p, numer, denom)
}

// Produces a curve point R such that R = P + Q
#[inline]
fn _add(p: &CurvePoint, q: &CurvePoint) -> CurvePoint {
    if p == q {
        return _double(&q);
    }

    let numer = Integer::from(&q.y - &p.y);
    let denom = Integer::from(&q.x - &p.x);

    _lambda(p, q, numer, denom)
}

// Performs elliptic curve point multiplication
// Note that this implementation uses the double-and-add method
// Thus, while faster than the Montgomery ladder method, it is not resistant to side-channel attacks
#[inline]
fn _mul(p: &CurvePoint, s: &Integer) -> CurvePoint {
    let mut q = p.clone();

    let m = s.significant_bits();
    let mut i = m - 2;

    loop {
        q = _double(&q);
        if s.get_bit(i as u32) {
            q += p;
        }

        if i == 0 {
            break;
        } else {
            i -= 1;
        }
    }

    q
}

impl<'a, 'b> Add<&'a CurvePoint> for &'b CurvePoint {
    type Output = CurvePoint;

    fn add(self, q: &'a CurvePoint) -> CurvePoint {
        assert_eq!(self.curve, q.curve);

        _add(&self, q)
    }
}

impl<'a> AddAssign<&'a CurvePoint> for CurvePoint {
    fn add_assign(&mut self, q: &'a CurvePoint) {
        assert_eq!(self.curve, q.curve);

        let result = _add(self, q);
        self.x = result.x;
        self.y = result.y;
    }
}

impl<'a, 'b> Mul<&'a Integer> for &'b CurvePoint {
    type Output = CurvePoint;

    fn mul(self, s: &'a Integer) -> CurvePoint {
        _mul(&self, s)
    }
}

impl<'a> MulAssign<&'a Integer> for CurvePoint {
    fn mul_assign(&mut self, s: &'a Integer) {
        let result = _mul(self, s);
        self.x = result.x;
        self.y = result.y;
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "({}, {})",
            self.x.to_string_radix(16),
            self.y.to_string_radix(16)
        )
    }
}

impl Display for CurvePoint {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "({}, {})",
            self.x.to_string_radix(16),
            self.y.to_string_radix(16)
        )
    }
}
