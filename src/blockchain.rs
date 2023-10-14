use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use hex;

/// use a 32-byte array (fixed length) store the `HashString`, so that
/// this value can be stored on heap
type HashValue = [u8; 32];

/// convert a `HashString` to a `String` with `0x` prefix
pub fn bytes_to_str(hash_value_bytes: HashValue) -> String {
    format!("0x{}", hex::encode(hash_value_bytes))
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    index: u64,
    data: String,
    timestamp: String,
    prev_hash: HashValue,
    hash: Option<HashValue>,
    root: HashValue,
    // Merkle Tree Root
    // 0 ~ 255, leading 0s of hash value (binary format)
    difficulty: u8,
    nonce: i64,
}

impl Block {
    pub fn sha256_digest(&self) -> HashValue {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_be_bytes());
        hasher.update(self.data.as_bytes());
        hasher.update(self.timestamp.as_bytes());
        hasher.update(self.prev_hash);
        hasher.update(self.root);
        hasher.update(self.difficulty.to_be_bytes());
        hasher.update(self.nonce.to_be_bytes());
        hasher.finalize().into()
    }
}

/// The most frequent operation on a blockchain is Query
/// Therefore, use a HashMap store the Blocks
/// key is the `index:[u8;32]` of a block, value is the `block:Block`
pub struct Blockchain {
    blockchain: Box<HashMap<HashValue, Block>>,
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;

    use crate::blockchain::{Block, bytes_to_str};

    #[test]
    fn test1() {
        let block = Block {
            index: 0,
            data: "Hello, World".to_string(),
            timestamp: "2021-10-01 00:00:00".to_string(),
            prev_hash: [0; 32],
            hash: None,
            root: [0; 32],
            difficulty: 0,
            nonce: 0,
        };
        println!("{}", bytes_to_str(block.sha256_digest()));
    }
}