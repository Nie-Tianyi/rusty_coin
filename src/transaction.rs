use crate::types::HashValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    inputs: Vec<Input>,        // 交易输入
    outputs: Vec<Output>,      // 交易输出
    transaction_id: HashValue, // 交易哈希值
    transaction_fee: f64,      // 交易费用
    additional_data: Vec<u8>,  // 附加数据
}
impl Transaction {
    pub fn new(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        transaction_id: HashValue,
        transaction_fee: f64,
        additional_data: Vec<u8>,
    ) -> Self {
        Self {
            inputs,
            outputs,
            transaction_id,
            transaction_fee,
            additional_data,
        }
    }
}
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct Input {
    transaction_hash: HashValue,  // 前一个交易的哈希值，也是前一个交易的ID
    previous_block_index: u64,    // 前一个交易所在的区块编号
    output_index: u64,            // 前一个交易中的第几个输出
    length_of_unlock_script: u64, // 签名的长度
    unlock_script: Vec<u8>,       // 解锁脚本: 发送者对上一个交易的签名 + 发送者公钥
}
impl Input {
    pub fn new(
        transaction_hash: HashValue,
        previous_block_index: u64,
        output_index: u64,
        unlock_script: Vec<u8>,
    ) -> Self {
        let length_of_unlock_script = unlock_script.len() as u64;
        Self {
            transaction_hash,
            previous_block_index,
            output_index,
            length_of_unlock_script,
            unlock_script,
        }
    }
}
#[derive(Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Output {
    amount: f64,                   // 金额
    length_of_locking_script: u64, // 锁定脚本的长度
    locking_script: Vec<u8>,       // 锁定脚本：接收者的公钥的哈希值
}
impl Output {
    pub fn new(amount: f64, locking_script: Vec<u8>) -> Self {
        let length_of_locking_script = locking_script.len() as u64;
        Self {
            amount,
            length_of_locking_script,
            locking_script,
        }
    }
}
