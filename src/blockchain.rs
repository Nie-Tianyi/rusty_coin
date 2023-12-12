use crate::transaction::Transaction;
use crate::types::HashValue;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Block {
    version: f64,           // 版本号
    index: u64,             // 区块编号
    timestamp: u64,         // UTF格式，自1970年1月1日以来的秒数
    prev_hash: HashValue,   // 前一个区块的哈希值
    hash: HashValue,        // 当前区块的哈希值，工作量证明
    merkle_root: HashValue, // 默克尔树根节点的哈希值
    difficulty: u32,        // 难度系数，每 1024 个区块调整一次难度, nBits的格式
    nonce: i64,             // 随机数，工作量证明
    data: Vec<Transaction>, // 交易数据
}
#[allow(clippy::identity_op)]
impl Block {
    /// calculate the target difficulty by the nBits in `difficulty`
    ///
    /// $ target\ threshold = b_2b_3b_4 \times 2^{8(b_1 - 3)} $
    pub fn target_threshold(&self) -> HashValue {
        let n_bit_bytes: [u8; 4] = self.difficulty.to_be_bytes();
        let mut target = [0u8; 32];
        let exp: usize = (n_bit_bytes[0] - 3) as usize;

        let mut i = 0usize;
        while i < 32 {
            if i == 32 - exp - 2 - 1 {
                target[i] = n_bit_bytes[1];
            } else if i == 32 - exp - 1 - 1 {
                target[i] = n_bit_bytes[2];
            } else if i == 32 - exp - 0 - 1 {
                target[i] = n_bit_bytes[3];
            } else {
                target[i] = 0x00;
            }
            i += 1;
        }

        HashValue::new(target)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blockchain: Vec<Block>,
}

fn create_genesis_block() -> Block {
    let current_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(e) => {
            println!("SystemTimeError: {}", e);
            1702312206u64
        }
    };

    Block {
        version: 0.1f64,
        index: 0,
        data: Vec::new(),
        timestamp: current_time,
        prev_hash: HashValue::new([0; 32]),
        hash: HashValue::new([0; 32]),
        merkle_root: HashValue::new([0; 32]),
        difficulty: 0,
        nonce: 0,
    }
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = create_genesis_block();
        Self {
            blockchain: vec![genesis_block],
        }
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_threshold() {
        let block = Block {
            version: 0.1f64,
            index: 0,
            data: Vec::new(),
            timestamp: 0u64,
            prev_hash: HashValue::new([0; 32]),
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty: 0x04123456u32,
            nonce: 0,
        };
        let target_threshold = block.target_threshold();

        assert_eq!(
            target_threshold.to_string(),
            "0x0000000000000000000000000000000000000000000000000000000012345600"
        );
    }
    #[test]
    fn test_create_genesis_block() {
        let genesis_block = create_genesis_block();
        println!("{:?}", genesis_block);
    }

    #[test]
    fn test_new_blockchain() {
        let blockchain = Blockchain::new();
        println!("{:?}", blockchain);
    }
}
