use crate::transaction::Transaction;
use crate::types::HashValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Block {
    version: f64,           // 版本号
    index: u64,             // 区块编号
    data: Vec<Transaction>, // 交易数据
    timestamp: u64,         // UTF格式，自1970年1月1日以来的秒数
    prev_hash: HashValue,   // 前一个区块的哈希值
    hash: HashValue,        // 当前区块的哈希值，工作量证明
    merkle_root: HashValue, // 默克尔树根节点的哈希值
    difficulty: u8,         // 难度系数，每 1024 个区块调整一次难度, nBits的格式
    nonce: i64,             // 随机数，工作量证明
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blockchain: Vec<Block>,
}
