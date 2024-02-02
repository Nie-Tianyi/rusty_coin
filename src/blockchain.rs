use crate::block::Block;
/// The core part of rusty coin
/// The mining rule of rusty coin:
///     - 10 seconds per block, adjust difficulty every hour
///     - the first transaction in every block should be the coinbase transaction
///     - coinbase need get 6 * 24 (= 1 day) confirmation before spent
///     - UTXO need get 6 (= 1 min) confirmation before spent
/// The reward rule of rusty coin is a convergent infinite geometric series:
///     - $ reward = $
use crate::transaction::{Output, Transaction};
use crate::types::HashValue;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    blockchain: Vec<Block>,    // store the blockchain / pieces of the blockchain
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

    pub fn filter_transactions_by_algo<F>(&self, algorithm: F) -> Vec<Transaction>
    where
        F: FnOnce(&Vec<Transaction>) -> Vec<Transaction>,
    {
        algorithm(&self.tx_pool)
    }

    /// get the latest reward of the blockchain
    pub fn get_latest_reward(&self, unpacked_transactions: &[Transaction]) -> Decimal {
        let aggregate_tx_fee = unpacked_transactions
            .iter()
            .fold(dec!(0.0), |sum, tx| sum + tx.get_transaction_fee());
        let reward = Self::reward_algorithm(self.get_last_block().unwrap().index + 1);
        reward + aggregate_tx_fee
    }

    /// generate a new block, including the coinbase transaction.
    ///
    /// this function include the mining process, which is time consuming
    /// # Arguments:
    /// * `address`: HashValue - the address of the miner
    /// * `protocol_version`: String - the version of the protocol
    /// * `time_millis`: u64 - the timestamp of the block
    /// * `prev_block`: &Block - the previous block
    /// * `difficulty`: u32 - the difficulty of the block
    /// * `tx_sorting_algo`: F - the sorting algorithm of the transactions
    ///    - the sorting algorithm may sort the transactions by their transaction fee
    pub fn generate_new_block(
        &self,
        receivers: Vec<(HashValue, Decimal)>,
        protocol_version: String,
        time_millis: u64,
        difficulty: u32,
        unpacked_transactions: Vec<Transaction>,
    ) -> Block {
        let prev_block = self.get_last_block().unwrap();
        // if the output is not valid, then panic
        let output_fee_sum = receivers.iter().fold(dec!(0.0), |sum, (_address, amount)| {
            if *amount < dec!(0.0) {
                panic!("Invalid output amount");
            }
            sum + amount
        });
        if output_fee_sum > Self::reward_algorithm(prev_block.index + 1) {
            panic!("Invalid output amount");
        }

        let coinbase_transaction = Self::create_coinbase_transaction(receivers);
        let mut unpacked_transactions = unpacked_transactions;
        unpacked_transactions.insert(0, coinbase_transaction);
        let mut block = Block {
            version: protocol_version,
            index: prev_block.index + 1,
            data: unpacked_transactions,
            timestamp: time_millis,
            prev_hash: prev_block.hash,
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty,
            nonce: 0,
        };
        block.merkle_root = block.calc_merkle_root();
        block.hash = block.find_valid_hash(); // POW algorithm, 2 rounds of sha256
        block
    }

    fn create_coinbase_transaction(receivers: Vec<(HashValue, Decimal)>) -> Transaction {
        let reward_outputs = receivers
            .into_iter()
            .map(|(address, amount)| Output::new(amount, address.to_vec()))
            .collect::<Vec<Output>>();
        let mut res = Transaction::new(
            vec![],
            reward_outputs,
            HashValue::new([0u8; 32]),
            dec!(0.0),
            None,
        );
        res.update_digest(); // update coinbase transaction's digest (transaction_id, hash value of the transaction)
        res
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
    /// - check all regular transactions in the block
    /// - check the coinbase transaction
    ///     - if it follows the reward rule of this blockchain
    ///     - if it equals to the sum of transaction fee
    /// - check the merkle root of the block
    /// - check the difficulty of the block
    /// - check the hash value of the block
    /// - check the timestamp of the block
    pub fn verify_block(&self, block: &Block, network_difficulty: u32) -> bool {
        self.verify_transactions(&block.data, block.index)
            && self.verify_merkle_root(block)
            && self.verify_difficulty(block, network_difficulty)
            && self.verify_block_hash(block)
            && self.verify_timestamp(block.timestamp)
    }

    pub fn verify_transactions(&self, transactions: &[Transaction], block_index: usize) -> bool {
        transactions.iter().all(|tx| {
            if Transaction::is_coinbase_transaction(tx) {
                self.verify_coinbase_transaction(tx, transactions, block_index)
            } else {
                self.verify_regular_transaction(tx)
            }
        })
    }

    /// verify a coinbase transaction's integrity, check if it is valid.
    /// - check if the transaction hash is equal to the transaction ID
    /// - check if the reward is valid
    ///
    /// # Arguments:
    /// * `coinbase_tx`: &Transaction - the coinbase transaction to be verified
    /// * `transactions`: &[Transaction] - all the transactions in the block
    /// * `block_index`: usize - the index of the block
    /// returns: bool - if the coinbase transaction is valid, return true, else return false
    fn verify_coinbase_transaction(
        &self,
        coinbase_tx: &Transaction,
        transactions: &[Transaction],
        block_index: usize,
    ) -> bool {
        let aggregate_tx_fee = transactions
            .iter()
            .skip(1) // skip the coinbase transaction
            .fold(dec!(0.0), |sum, tx| sum + tx.get_transaction_fee());

        // check if the transaction hash is equal to the transaction ID
        if dbg!(coinbase_tx.sha256() != coinbase_tx.get_transaction_id()) {
            return false;
        }
        // check if the reward is valid
        let reward = Self::reward_algorithm(block_index);

        let mut output_fee_sum = dec!(0.0);
        for output in coinbase_tx.get_outputs() {
            if output.get_amount() < dec!(0.0) {
                return false;
            }
            output_fee_sum += output.get_amount();
        }

        output_fee_sum <= reward + aggregate_tx_fee
    }

    fn verify_merkle_root(&self, block: &Block) -> bool {
        block.merkle_root == block.calc_merkle_root()
    }

    fn verify_difficulty(&self, block: &Block, network_difficulty: u32) -> bool {
        block.difficulty == network_difficulty
    }

    fn verify_block_hash(&self, block: &Block) -> bool {
        block.sha256().sha256() == block.hash && block.hash <= block.target_threshold()
    }

    /// verify timestamp (in seconds, UTC +0:00) of the block:
    /// the timestamp of the block should be less than the current time
    /// and the timestamp of the block should be larger than the average timestamps of the previous 10 blocks
    /// (if there are less than 10 blocks, then use the average timestamps of all the previous blocks)
    fn verify_timestamp(&self, timestamp: u64) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut average_timestamp = 0_u64;
        let mut count = 0;
        for block in self.blockchain.iter().rev() {
            if count >= 10 {
                break;
            }
            average_timestamp += block.timestamp;
            count += 1;
        }
        average_timestamp /= count;
        dbg!(average_timestamp) < dbg!(timestamp) && timestamp < dbg!(current_time)
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
    ///
    /// # Arguments:
    /// * `transaction`: &Transaction - the transaction to be verified
    ///
    /// returns: bool - if the transaction is valid, return true, else return false
    fn verify_regular_transaction(&self, transaction: &Transaction) -> bool {
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

    /// reward rule of the coinbase transaction
    /// - the reward of the coinbase transaction is a convergent infinite series
    /// - the reward of the coinbase transaction is 50 rusty coin at the beginning
    fn reward_algorithm(index: usize) -> Decimal {
        let inverse_log = 18.0 * 1.0 / (index.to_f64().unwrap() + 1.0_f64).log10();
        let inverse_log_floor = inverse_log.floor();
        Decimal::from_f64(inverse_log_floor).unwrap()
    }

    /// create the first block of a blockchain, the genesis block
    ///
    /// # Arguments
    /// * `init_msg: &str` - the message of the genesis transaction of the genesis block
    fn create_genesis_block(init_msg: &str) -> Block {
        let init_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(e) => {
                panic!("SystemTimeError: {}", e);
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
            version: "0.1v test".to_string(),
            index: 0,
            data: vec![genesis_transaction],
            timestamp: init_time,
            prev_hash: HashValue::new([0; 32]),
            hash: HashValue::new([0; 32]),
            merkle_root: HashValue::new([0; 32]),
            difficulty: 0,
            nonce: 0,
        };

        genesis_block.merkle_root = genesis_block.calc_merkle_root(); // update merkle root of the genesis block
        genesis_block.hash = genesis_block.sha256(); // update hash value of the genesis block
        genesis_block
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new("Default Blockchain")
    }
}

impl Display for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[Blockchain]:")?;
        for block in self.blockchain.iter() {
            write!(f, "{}\n\n\n", block)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_new_blockchain() {
        let blockchain = Blockchain::new("hello world");
        println!("{:?}", blockchain);
    }

    #[test]
    fn test_reward_algorithm() {
        let reward = Blockchain::reward_algorithm(1844674407370955161);
        println!("{}", reward);
    }

    #[test]
    fn test_generate_new_block() {
        let mut blockchain = Blockchain::new("hello world");
        let latest_reward_fee = blockchain.get_latest_reward(&[]);
        sleep(std::time::Duration::from_secs(1)); // simulate the time before creating a new block
        let block = blockchain.generate_new_block(
            vec![(HashValue::new([0u8; 32]), latest_reward_fee)],
            "0.1v test".to_string(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(), // for the test, set the timestamp to 10 seconds later
            0x1E123456_u32,
            vec![],
        );
        sleep(std::time::Duration::from_secs(1)); //simulate the time gap between mining and verifying process
        let res = blockchain.verify_block(&block, 0x1E123456_u32);
        assert!(res);

        blockchain.add_block(block);
        println!("{}", blockchain);
    }
}
