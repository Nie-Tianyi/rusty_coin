use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum RustyCoinError {
    InvalidOutputIndex,
    InvalidInputFee,
    InvalidBlockIndex,
    InvalidTransactionIndex,
}

impl Display for RustyCoinError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RustyCoinError::InvalidOutputIndex => {
                write!(f, "The output index is invalid, Invalid Output")
            }
            RustyCoinError::InvalidInputFee => write!(
                f,
                "The inputs' fee is larger than outputs' fee, Invalid Inputs"
            ),
            RustyCoinError::InvalidBlockIndex => write!(f, "Invalid block index"),
            RustyCoinError::InvalidTransactionIndex => write!(f, "Invalid transaction index"),
        }
    }
}

impl Error for RustyCoinError {}
