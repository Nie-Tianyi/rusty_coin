use std::error::Error;
use std::fmt::{Display, Formatter};
// This enum only store the error that could cause a system failure
#[derive(Debug)]
pub enum RustyCoinError {
    InvalidOutputIndex,
    InvalidInputFee,
    InvalidBlockIndex,
    InvalidTransactionIndex,
    InvalidOutputAmount,
}

#[derive(Debug)]
pub struct InvalidTransactionInput(String);

impl Display for InvalidTransactionInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid transaction input")
    }
}

impl Error for InvalidTransactionInput {}

#[derive(Debug)]
pub struct InvalidTransactionOutput(String);

impl Display for InvalidTransactionOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid transaction input")
    }
}

impl Error for InvalidTransactionOutput {}
