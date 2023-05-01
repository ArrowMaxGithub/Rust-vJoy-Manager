use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub struct ShiftModeMask(pub u8);
impl Display for ShiftModeMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:b}", self.0))
    }
}
