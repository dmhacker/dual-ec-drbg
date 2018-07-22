use ramp::int::Int;
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct CurvePoint {
    pub x: Int, 
    pub y: Int 
}

impl Display for CurvePoint {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}, {})", self.x.to_str_radix(16, false), self.y.to_str_radix(16, false))
    }
}
