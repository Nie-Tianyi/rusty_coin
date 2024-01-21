/// The core part of rusty coin
/// The mining rule of rusty coin:
///     - 10 seconds per block, adjust difficulty every hour
///     - the first transaction in every block should be the coinbase transaction
///     - coinbase need get 6 * 24 (= 1 day) confirmation before spent
///     - UTXO need get 6 (= 1 min) confirmation before spent
/// The reward rule of rusty coin:
///     - $ reward = $
use crate::transaction::Transaction;
use crate::types::HashValue;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// The `Block` struct represents a block in the blockchain.
///
/// # Fields
///
/// * `version` - A floating point number representing the version of the block.
/// * `index` - An unsigned 64-bit integer representing the block number.
/// * `timestamp` - An unsigned 64-bit integer representing the time the block was created, in seconds since the Unix Epoch (January 1, 1970).
/// * `prev_hash` - A `HashValue` representing the hash of the previous block in the blockchain.
/// * `hash` - A `HashValue` representing the hash of the current block. This is the proof of work.
/// * `merkle_root` - A `HashValue` representing the root hash of the Merkle tree of the transactions included in the block.
/// * `difficulty` - An unsigned 32-bit integer representing the difficulty target for the proof of work. The difficulty is adjusted every 1024 blocks.
/// * `nonce` - A signed 64-bit integer used in the proof of work.
/// * `data` - A vector of `Transaction` structs representing the transactions included in the block.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    version: f64,           // version of the block
    index: usize,           // block height
    timestamp: u64,         // time elapsed since the Unix Epoch (January 1, 1970) in seconds
    prev_hash: HashValue,   // previous block hash
    hash: HashValue,        // hash value of current block
    merkle_root: HashValue, // merkle root of all the transactions
    difficulty: u32,        // difficulty target for the proof of work, adjusted every 1024 blocks
    nonce: i64,             // random number
    data: Vec<Transaction>, // transactions
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
    pub fn merkle_root(&self) -> HashValue {
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
    /// find the valid hash value by the proof of work
    pub fn find_valid_hash(&mut self) -> HashValue {
        let mut nonce = self.nonce;
        let target_threshold = self.target_threshold();
        let mut valid_hash = self.sha256().sha256();

        while valid_hash > target_threshold {
            nonce += 1;
            self.nonce = nonce;
            valid_hash = self.sha256().sha256();
            // println!("nonce: {}, hash: {}", nonce, valid_hash)
        }
        valid_hash
    }

    /// calculate the hash value of the block
    fn sha256(&self) -> HashValue {
        let mut hasher = Sha256::new();
        hasher.update(self.version.to_be_bytes());
        hasher.update(self.index.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.prev_hash);
        hasher.update(self.merkle_root);
        hasher.update(self.difficulty.to_be_bytes());
        hasher.update(self.nonce.to_be_bytes());
        let result = hasher.finalize().into();
        HashValue::new(result)
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `tx_id`: HashValue - the hash value of the transaction
    ///
    /// returns: Option<&Transaction>
    fn get_tx_by_id(&self, tx_id: HashValue) -> Option<&Transaction> {
        self.data.iter().find(|tx| tx.get_transaction_id() == tx_id)
    }
}

impl Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let hash = self.sha256();
        hash.iter().for_each(|byte| state.write_u8(*byte));
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    blockchain: Vec<Block>,    // store the blockchain
    tx_pool: Vec<Transaction>, // store the unpacked transactions
}

impl Blockchain {
    /// create a new blockchain, including the genesis block
    pub fn new(genesis_msg: &str) -> Self {
        let genesis_block = Self::create_genesis_block(genesis_msg);
        Self {
            blockchain: vec![genesis_block],
            tx_pool: vec![],
        }
    }

    /// add a block to the blockchain
    /// * please verify the block before calling this function!!!
    /// * please verify the block before calling this function!!!
    /// * please verify the block before calling this function!!!
    pub fn add_block(&mut self, block: Block) {
        self.blockchain.push(block);
    }

    /// get the block by its index
    pub fn get_block(&self, index: usize) -> Option<&Block> {
        self.blockchain.get(index)
    }

    /// get the latest block of the blockchain
    pub fn get_last_block(&self) -> Option<&Block> {
        self.blockchain.last()
    }

    /// verify a block's integrity, check if it is valid
    /// - check all the transaction in the block
    /// - check the coinbase transaction
    ///     - if it follow the reward rule of this blockchain
    ///     - if it equal to the sum of transaction fee
    /// - check the merkle root of the block
    /// - check the difficulty of the block
    /// - check the hash value of the block
    /// - check the timestamp of the block
    pub fn verify_block(&self, block: &Block) -> bool {
        let target_threshold = block.target_threshold();
        block.hash <= target_threshold
    }

    /// verify a regular transaction's integrity, check if it is valid.
    ///
    /// if it is a coinbase transaction, please use `fn verify_coinbase_transaction` instead.
    /// - check if the transaction hash is equal to the transaction ID
    /// - check if the transaction fee is valid
    /// - check if the inputs are legal, the inputs should be unspent
    ///     - check if the unlock script is valid
    ///     - check if the previous transaction hash is valid
    ///     - check if the previous output index is valid
    pub fn verify_transaction(&self, transaction: &Transaction) -> bool {
        // check if inputs are legal:
        // - check prev_transaction_hash
        // - check unlocking_script
        let mut input_fee_sum = dec!(0.0);
        for input in transaction.get_inputs() {
            // get previous block that contain this input tx, if it is None, then return false
            let block = match self.blockchain.get(input.get_prev_block_index()) {
                Some(block) => block,
                None => {
                    return false;
                }
            };
            // get the previous transaction, if it is None, then return false
            let prev_tx = match block.get_tx_by_id(input.get_prev_tx_hash()) {
                Some(prev_tx) => prev_tx,
                None => {
                    return false;
                }
            };
            // get the output, if the index is invalid, then return false
            let prev_output = match prev_tx.get_output_by_index(input.get_prev_output_index()) {
                Some(prev_output) => prev_output,
                None => {
                    return false;
                }
            };

            input_fee_sum += prev_output.get_amount();
            // verify the unlock script
            if !Transaction::verify_scripts(
                prev_tx,
                prev_output.get_locking_script(),
                input.get_unlock_script(),
            ) {
                return false;
            }
        }
        // check the transaction hash, if it is equal to the transaction ID
        if transaction.get_transaction_id() != transaction.sha256() {
            return false;
        }
        // check if the transaction fee is valid
        // calculate the output sum
        for output in transaction.get_outputs() {
            // all the output amount should be positive (>= 0)
            if output.get_amount() < dec!(0.0) {
                return false;
            }
            input_fee_sum -= output.get_amount();
        }
        if transaction.get_transaction_fee() != input_fee_sum {
            return false;
        }

        true
    }

    fn create_genesis_block(init_msg: &str) -> Block {
        let init_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(e) => {
                println!("SystemTimeError: {}", e);
                0u64
            }
        };

        let mut genesis_transaction = Transaction::new(
            vec![],
            vec![],
            HashValue::new([0u8; 32]),
            dec!(0.0),
            Some(init_msg.as_bytes().to_vec()),
        );

        genesis_transaction.update_digest(); // update genesis transaction's digest (transaction_id, hash value of the transaction)

        let mut genesis_block = Block {
            version: 0.1f64,
            index: 0,
            data: vec![genesis_transaction],
            timestamp: init_time,
            prev_hash: HashValue::new([0; 32]),
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty: 0,
            nonce: 0,
        };

        genesis_block.merkle_root = genesis_block.merkle_root(); // update merkle root of the genesis block
        genesis_block.hash = genesis_block.sha256(); // update hash value of the genesis block
        genesis_block
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new("Default Blockchain")
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
            difficulty: 0x1E123456u32,
            nonce: 0,
        };
        let target_threshold = block.target_threshold();

        assert_eq!(
            target_threshold.to_string(),
            "0x0000123456000000000000000000000000000000000000000000000000000000"
        );
    }
    #[test]
    fn test_merkle_root() {
        let block = Block {
            version: 0.1f64,
            index: 0,
            data: vec![
                Transaction::new(vec![], vec![], HashValue::new([0u8; 32]), dec!(0.0), None),
                Transaction::new(vec![], vec![], HashValue::new([0u8; 32]), dec!(0.0), None),
            ],
            timestamp: 0u64,
            prev_hash: HashValue::new([0; 32]),
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty: 0x04123456u32,
            nonce: 0,
        };
        let merkle_root = block.merkle_root();
        println!("{}", merkle_root);
    }
    #[test]
    fn test_block_sha256() {
        let block = Block {
            version: 0.1f64,
            index: 0,
            data: Vec::new(),
            timestamp: 0u64,
            prev_hash: HashValue::new([0; 32]),
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty: 0x04123456u32,
            nonce: 143,
        };

        let hash = block.sha256().sha256();
        assert_eq!(
            hash.to_string(),
            "0x8ea07e6d3035f172f8d81c803dd65516b2dfd5af9da892dd0ae554bff6ba7a59"
        )
    }

    #[test]
    fn test_new_blockchain() {
        let blockchain = Blockchain::new("hello world");
        println!("{:?}", blockchain);
    }

    #[test]
    fn test_mining() {
        let mut block = Block {
            version: 0.1f64,
            index: 0,
            data: Vec::new(),
            timestamp: 0u64,
            prev_hash: HashValue::new([0; 32]),
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty: 0x1E123456u32,
            nonce: 2054888,
        };
        block.hash = block.find_valid_hash();
        println!("{:?}", block);
    }
}
