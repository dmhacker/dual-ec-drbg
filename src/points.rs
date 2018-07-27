use rug::Integer; 
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct CurvePoint {
    pub x: Integer, 
    pub y: Integer
}

impl Display for CurvePoint {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}, {})", self.x.to_string_radix(16), self.y.to_string_radix(16))
    }
}
