use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum RustyCoinError {
    InvalidOutputIndex,
    InvalidInputFee,
}

impl Display for RustyCoinError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RustyCoinError::InvalidOutputIndex => write!(f, "The Output length is larger than you system usize, fail indexing Outputs of this transaction"),
            RustyCoinError::InvalidInputFee => write!(f, "The inputs' fee is larger than outputs' fee, Invalid Inputs"),
        }
    }
}

impl Error for RustyCoinError {}
