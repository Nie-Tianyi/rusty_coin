use std::collections::HashMap;
use chrono::{DateTime, Local};

type HashValue = [u8; 32];

/// convert a `HashString` to a `String` with `0x` prefix
pub struct TransactionPool {
    pool: Box<HashMap<HashValue, Transaction>>,
}

pub struct Transaction {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    timestamp: String,
    hash: HashValue,
}

pub struct Input {
    transaction_hash: HashValue,
    index: u64,
    signature: HashValue,
}

pub struct Output {
    receiver: HashValue,
    amount: f64,
}

