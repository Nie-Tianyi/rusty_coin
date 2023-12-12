use crate::types::HashValue;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Represents a transaction in the blockchain.
#[derive(Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    inputs: Vec<Input>,        // The inputs for the transaction.
    outputs: Vec<Output>,      // The outputs for the transaction.
    transaction_id: HashValue, // The unique identifier for the transaction.
    transaction_fee: f64,      // The fee associated with the transaction.
    additional_data: Vec<u8>,  // Any additional data associated with the transaction.
}

impl Transaction {
    /// Creates a new transaction.
    ///
    /// # Arguments
    ///
    /// * `inputs` - A vector of inputs for the transaction.
    /// * `outputs` - A vector of outputs for the transaction.
    /// * `transaction_id` - A unique identifier for the transaction.
    /// * `transaction_fee` - The fee associated with the transaction.
    /// * `additional_data` - Any additional data associated with the transaction.
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
    /// Calculates the SHA256 hash of the transaction.
    ///
    /// # Returns
    ///
    /// * A `HashValue` representing the SHA256 hash of the transaction.
    pub fn sha256(&self) -> HashValue {
        let mut hasher = Sha256::new();

        for input in &self.inputs {
            hasher.update(input.transaction_hash);
            hasher.update(input.previous_block_index.to_be_bytes());
            hasher.update(input.output_index.to_be_bytes());
            hasher.update(input.length_of_unlock_script.to_be_bytes());
            hasher.update(&input.unlock_script);
        }

        for output in &self.outputs {
            hasher.update(output.amount.to_be_bytes());
            hasher.update(output.length_of_locking_script.to_be_bytes());
            hasher.update(&output.locking_script);
        }

        hasher.update(self.transaction_id);
        hasher.update(self.transaction_fee.to_be_bytes());
        hasher.update(&self.additional_data);
        let result = hasher.finalize().into();
        HashValue::new(result)
    }
}

/// Represents an input for a transaction.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct Input {
    transaction_hash: HashValue,  // The hash of the previous transaction.
    previous_block_index: u64,    // The index of the previous block.
    output_index: u64,            // The index of the output in the previous transaction.
    length_of_unlock_script: u64, // The length of the unlock script.
    unlock_script: Vec<u8>,       // The unlock script: a signature and a public key.
}

impl Input {
    /// Creates a new input for a transaction.
    ///
    /// # Arguments
    ///
    /// * `transaction_hash` - The hash of the previous transaction.
    /// * `previous_block_index` - The index of the previous block.
    /// * `output_index` - The index of the output in the previous transaction.
    /// * `unlock_script` - The unlock script. This is a signature and a public key.
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

/// Represents an output for a transaction.
#[derive(Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Output {
    amount: f64,                   // The amount of the output.
    length_of_locking_script: u64, // The length of the locking script.
    locking_script: Vec<u8>,       // The locking script: a public key hash.
}

impl Output {
    /// Creates a new output for a transaction.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount of the output.
    /// * `locking_script` - The locking script. This is a public key hash.
    pub fn new(amount: f64, locking_script: Vec<u8>) -> Self {
        let length_of_locking_script = locking_script.len() as u64;
        Self {
            amount,
            length_of_locking_script,
            locking_script,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hasher() {
        let transaction = Transaction::new(
            vec![Input::new(HashValue::new([0u8; 32]), 0, 0, vec![0u8; 32])],
            vec![Output::new(0.0, vec![0u8; 32])],
            HashValue::new([0u8; 32]),
            0.0,
            vec![1u8; 32],
        );

        println!("{}", serde_json::to_string(&transaction).unwrap());

        let hash = transaction.sha256(); //hash once

        assert_eq!(
            hash.to_string(),
            "0x153c2bcc69f5f5fd2ddbe53b9ffd6fdeddf376dc9eb49ef4e59f024131a5d1f5"
        );

        let hash = transaction.sha256().sha256(); //hash twice

        assert_eq!(
            hash.to_string(),
            "0x0de448964b46b5530f4936f728fabf562c14a181acb7526317ccbe3986b5774d"
        );
    }
}
