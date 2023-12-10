use crate::types::Bytes;
use serde::{Deserialize, Serialize};

type HashValue = Bytes<32>;

#[allow(dead_code)]
pub struct TransactionPool {
    pool: Vec<Transaction>,
}
#[derive(Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    transaction_id: HashValue,
    transaction_fee: f64,
}
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct Input {
    transaction_hash: HashValue, // 前一个交易的哈希值
    index: u64,                  // 前一个交易中的第几个输出
    response_script: Bytes<64>,
}
#[derive(Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Output {
    amount: f64,
    challenge_script: Bytes<64>,
}
