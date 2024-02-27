use crate::transaction::Transaction;
use crate::types::HashValue;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Display;

/// The `Block` struct represents a block in the blockchain.
///
/// tips: consider use log crate to print log
/// # Fields
///
/// * `version` - A floating point number representing the version of the block.
/// * `index` - An unsigned 64-bit integer representing the block number.
/// * `timestamp` - An unsigned 64-bit integer representing the time the block was created, in seconds since the Unix Epoch (January 1, 1970).
/// * `prev_hash` - A `HashValue` representing the hash of the previous block in the blockchain.
/// * `hash` - A `HashValue` representing the hash of the current block. This is the proof of work.
/// * `merkle_root` - A `HashValue` representing the root hash of the Merkle tree of the transactions included in the block.
/// * `difficulty` - An unsigned 32-bit integer (in nBits format) representing the difficulty target for the proof of work. The difficulty is adjusted every block.
/// * `nonce` - A signed 64-bit integer used in the proof of work.
/// * `data` - A vector of `Transaction` structs representing the transactions included in the block.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub(crate) version: String,        // version of the block
    pub(crate) index: usize,           // block height
    pub(crate) timestamp: u64, // time elapsed since the Unix Epoch (January 1, 1970) in seconds
    pub(crate) prev_hash: HashValue, // previous block hash
    pub(crate) hash: HashValue, // hash value of current block
    pub(crate) merkle_root: HashValue, // merkle root of all the transactions
    pub(crate) difficulty: u32, // difficulty target for the proof of work, adjusted every 1024 blocks
    pub(crate) nonce: i64,      // random number
    pub(crate) data: Vec<Transaction>, // transactions
}

impl Block {
    /// calculate the target difficulty by the nBits in `difficulty`
    ///
    /// $ target\ threshold = b_2b_3b_4 \times 2^{8(b_1 - 3)} $
    #[allow(clippy::identity_op)]
    pub fn target_threshold(&self) -> HashValue {
        let n_bit_bytes: [u8; 4] = self.difficulty.to_be_bytes();
        let mut target = [0u8; 32];
        let exp: isize = n_bit_bytes[0] as isize - 3;

        let mut i = 0usize;
        while i < 32 {
            if i == (32 - exp - 2 - 1) as usize {
                target[i] = n_bit_bytes[1];
            } else if i == (32 - exp - 1 - 1) as usize {
                target[i] = n_bit_bytes[2];
            } else if i == (32 - exp - 0 - 1) as usize {
                target[i] = n_bit_bytes[3];
            } else {
                target[i] = 0x00;
            }
            i += 1;
        }

        HashValue::new(target)
    }

    /// calculate the merkle root of all the transactions
    pub fn calc_merkle_root(&self) -> HashValue {
        // return 0x00...000 directly if the data is empty
        if self.data.is_empty() {
            return HashValue::new([0; 32]);
        }

        //calculate all the transactions' hash value
        let mut hashes = self
            .data
            .iter()
            .map(|transaction| transaction.sha256())
            .collect::<Vec<HashValue>>();

        // construct a merkle tree
        while hashes.len() > 1 {
            hashes = hashes
                .chunks(2)
                .map(|chunk| match *chunk {
                    [hash] => hash,
                    [hash1, hash2] => {
                        let mut hasher = Sha256::new();
                        hasher.update(hash1);
                        hasher.update(hash2);
                        let result = hasher.finalize().into();
                        HashValue::new(result)
                    }
                    _ => unreachable!(), // panic immediately if none of the previous pattern get matched
                })
                .collect::<Vec<HashValue>>();
        }

        hashes[0]
    }
    /// POW algorithm,
    /// find the valid hash value by the proof of work
    pub fn update_hash_and_nonce(&mut self) {
        let mut nonce = self.nonce;
        let target_threshold = self.target_threshold();
        let mut valid_hash = self.sha256().sha256();

        while valid_hash > target_threshold {
            nonce += 1;
            self.nonce = nonce;
            valid_hash = self.sha256().sha256();
            // println!("nonce: {}, hash: {}", nonce, valid_hash)
        }
        self.hash = valid_hash;
    }

    /// calculate the hash value of the block
    pub fn sha256(&self) -> HashValue {
        let mut hasher = Sha256::new();
        hasher.update(self.version.as_bytes());
        hasher.update(self.index.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.prev_hash);
        hasher.update(self.merkle_root);
        hasher.update(self.difficulty.to_be_bytes());
        hasher.update(self.nonce.to_be_bytes());
        let result = hasher.finalize().into();
        HashValue::new(result)
    }

    /// # Arguments
    ///
    /// * `tx_id`: HashValue - the hash value of the transaction
    ///
    /// returns: Option<&Transaction>
    pub fn get_tx_by_id(&self, tx_id: HashValue) -> Option<&Transaction> {
        self.data.iter().find(|tx| tx.get_transaction_id() == tx_id)
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Block[{}]:", self.index)?;
        writeln!(f, "\tversion: {}", self.version)?;
        writeln!(f, "\ttimestamp: {}", self.timestamp)?;
        writeln!(f, "\tprev_hash: {}", self.prev_hash)?;
        writeln!(f, "\thash: {}", self.hash)?;
        writeln!(f, "\tmerkle_root: {}", self.merkle_root)?;
        writeln!(f, "\tdifficulty: {}", self.difficulty)?;
        writeln!(f, "\tnonce: {}", self.nonce)?;
        writeln!(f, "\tdata: [")?;
        for tx in self.data.iter() {
            let tx_str = format!("{}", tx);
            for line in tx_str.lines() {
                writeln!(f, "\t\t{}", line)?;
            }
        }
        writeln!(f, "\t]")
    }
}

#[cfg(test)]
mod tests {
    use crate::block::Block;
    use crate::transaction::Transaction;
    use crate::types::HashValue;
    use rust_decimal_macros::dec;

    #[test]
    fn test_target_threshold() {
        let block = Block {
            version: "0.1v test".to_string(),
            index: 0,
            data: Vec::new(),
            timestamp: 0u64,
            prev_hash: HashValue::new([0; 32]),
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty: 0x20123456_u32,
            nonce: 0,
        };
        let target_threshold = block.target_threshold();

        assert_eq!(
            target_threshold.to_string(),
            "0x1234560000000000000000000000000000000000000000000000000000000000"
        );
    }
    #[test]
    fn test_merkle_root() {
        let block = Block {
            version: "0.1v test".to_string(),
            index: 0,
            data: vec![
                Transaction::new(vec![], vec![], HashValue::new([0u8; 32]), dec!(0.0), None),
                Transaction::new(vec![], vec![], HashValue::new([0u8; 32]), dec!(0.0), None),
            ],
            timestamp: 0_u64,
            prev_hash: HashValue::new([0; 32]),
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty: 0x04123456_u32,
            nonce: 0,
        };
        let merkle_root = block.calc_merkle_root();
        println!("{}", merkle_root);
    }
    #[test]
    fn test_block_sha256() {
        let block = Block {
            version: "0.1v test".to_string(),
            index: 0,
            data: Vec::new(),
            timestamp: 0_u64,
            prev_hash: HashValue::new([0; 32]),
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty: 0x04123456_u32,
            nonce: 143,
        };

        let hash = block.sha256().sha256();
        println!("{}", hash);
    }

    #[test]
    fn test_mining() {
        let mut block = Block {
            version: "0.1v test".to_string(),
            index: 0,
            data: Vec::new(),
            timestamp: 0_u64,
            prev_hash: HashValue::new([0; 32]),
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty: 0x1E123456_u32,
            nonce: 0,
        };
        block.update_hash_and_nonce();
        println!("{}", block);
    }
}
