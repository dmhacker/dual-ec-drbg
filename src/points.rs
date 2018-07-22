use num::bigint::BigInt;
use num::traits::Zero;
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct CurvePoint {
    pub x: BigInt, 
    pub y: BigInt 
}

impl CurvePoint {
    pub fn origin() -> CurvePoint {
        CurvePoint {
            x: Zero::zero(),
            y: Zero::zero()
        }
    }
}

impl Display for CurvePoint {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}, {})", self.x.to_str_radix(16), self.y.to_str_radix(16))
    }
}
