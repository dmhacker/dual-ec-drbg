use std::rc::Rc;
use std::fmt::{Display, Formatter, Result};
use std::ops::{Add, AddAssign, Mul, MulAssign};
use rug::Integer; 
use curves::Curve;
use math::ModExtensions;

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub x: Integer, 
    pub y: Integer
}

impl Point {
    pub fn convert(&self, curve : Rc<Curve>) -> CurvePoint {
        CurvePoint {
            x: self.x.clone(),
            y: self.y.clone(),
            curve: curve
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}, {})", self.x.to_string_radix(16), self.y.to_string_radix(16))
    }
}

impl<'a> From<&'a CurvePoint> for Point {
    fn from(point : &'a CurvePoint) -> Point {
        Point {
            x: point.x.clone(),
            y: point.y.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurvePoint {
    pub x: Integer, 
    pub y: Integer,
    pub curve: Rc<Curve>
}

impl Display for CurvePoint {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}, {})", self.x.to_string_radix(16), self.y.to_string_radix(16))
    }
}

#[inline]
fn _lambda(p : &CurvePoint, q : &CurvePoint, numer : Integer, mut denom : Integer) -> CurvePoint {
    denom.invert_mut(&p.curve.p).unwrap();

    let mut lambda = Integer::from(&numer * &denom);
    lambda.modulo_mut(&p.curve.p);
    
    let mut rx = Integer::from(&lambda * &lambda);
    rx -= &p.x;
    rx -= &q.x;
    rx.modulo_mut(&p.curve.p);

    let mut ry = Integer::from(&p.x - &rx);
    ry *= &lambda;
    ry -= &p.y;
    ry.modulo_mut(&p.curve.p);

    CurvePoint {
        x: rx,
        y: ry,
        curve: Rc::clone(&p.curve)
    }
}

#[inline]
fn _double(p : &CurvePoint) -> CurvePoint {
    let mut numer = Integer::from(3);
    numer *= &p.x;
    numer *= &p.x;
    numer += &p.curve.a;

    let mut denom = Integer::from(2);
    denom *= &p.y;

    _lambda(p, p, numer, denom)
}

#[inline]
fn _add(p : &CurvePoint, q : &CurvePoint) -> CurvePoint {
    if p == q {
        return _double(&q);
    }

    assert_eq!(p.curve, q.curve);
    
    let numer = Integer::from(&q.y - &p.y); 
    let denom = Integer::from(&q.x - &p.x);

    _lambda(p, q, numer, denom)
}

#[inline]
fn _mul(p : &CurvePoint, s : &Integer) -> CurvePoint {
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
        }
        else {
            i -= 1;
        }
    }

    q
}

impl<'a, 'b> Add<&'a CurvePoint> for &'b CurvePoint {
    type Output = CurvePoint;

    fn add(self, q : &'a CurvePoint) -> CurvePoint {
        _add(&self, q)
    }
}

impl<'a> AddAssign<&'a CurvePoint> for CurvePoint {
    fn add_assign(&mut self, q : &'a CurvePoint) {
        let result = _add(self, q);
        self.x = result.x;
        self.y = result.y;
    }
}

impl<'a, 'b> Mul<&'a Integer> for &'b CurvePoint {
    type Output = CurvePoint;

    fn mul(self, s : &'a Integer) -> CurvePoint {
        _mul(&self, s)
    }
}

impl<'a> MulAssign<&'a Integer> for CurvePoint {
    fn mul_assign(&mut self, s : &'a Integer) {
        let result = _mul(self, s);
        self.x = result.x;
        self.y = result.y;
    }
}
