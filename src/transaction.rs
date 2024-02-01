use crate::types::{bytes_vec_to_hex_string, HashValue};
use rust_decimal::Decimal;
use secp256k1::{ecdsa::Signature, Message, PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Display;
use std::hash::{Hash, Hasher};

/// Represents a transaction in the blockchain.
/// Pay2PubKeyHash(P2PKH) is used as the locking script.
#[derive(Debug, Eq, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    inputs: Vec<Input>,               // The inputs for the transaction.
    outputs: Vec<Output>,             // The outputs for the transaction.
    transaction_id: HashValue,        // hash value of this transaction.
    transaction_fee: Decimal,         // difference between inputs and outputs.
    additional_data: Option<Vec<u8>>, // Any additional data associated with the transaction.
}

impl Transaction {
    /// Creates a new transaction.
    ///
    /// # Arguments
    ///
    /// * `inputs` - A vector of inputs for the transaction.
    /// * `outputs` - A vector of outputs for the transaction.
    /// * `transaction_id` - A unique identifier for the transaction. (SHA256 hash value of the transaction)
    /// * `transaction_fee` - The fee associated with the transaction.
    /// * `additional_data` - Any additional data associated with the transaction.
    pub fn new(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        transaction_id: HashValue,
        transaction_fee: Decimal,
        additional_data: Option<Vec<u8>>,
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
            hasher.update(input.prev_transaction_hash);
            hasher.update(input.prev_block_index.to_be_bytes());
            hasher.update(input.prev_output_index.to_be_bytes());
            hasher.update(input.length_of_unlock_script.to_be_bytes());
            hasher.update(&input.unlock_script);
        }

        for output in &self.outputs {
            hasher.update(serde_json::to_vec(&output.amount).unwrap());
            hasher.update(output.length_of_locking_script.to_be_bytes());
            hasher.update(&output.locking_script);
        }

        hasher.update(self.transaction_id);
        hasher.update(serde_json::to_vec(&self.transaction_fee).unwrap());
        if self.additional_data.is_some() {
            hasher.update(&self.additional_data.clone().unwrap());
        }
        let result = hasher.finalize().into();
        HashValue::new(result)
    }

    /// update transaction ID of this transaction:
    /// * transaction id is the SHA256 of the transaction
    /// * calculate the SHA256 of this transaction, and assign it to the `transaction_id` field
    pub fn update_digest(&mut self) {
        self.transaction_id = self.sha256();
    }
    /// get one output from the outputs vector
    /// # Arguments
    /// * `index` - the index of the output
    /// # Returns
    /// * `Some(output)` - if the index is valid
    pub fn get_output_by_index(&self, index: usize) -> Option<&Output> {
        self.outputs.get(index)
    }
    /// get the transaction fee of this transaction
    /// # Returns
    /// * `transaction_fee` - the transaction fee of this transaction
    pub fn get_transaction_fee(&self) -> Decimal {
        self.transaction_fee
    }
    pub fn get_inputs(&self) -> &Vec<Input> {
        &self.inputs
    }
    pub fn get_outputs(&self) -> &Vec<Output> {
        &self.outputs
    }
    pub fn get_transaction_id(&self) -> HashValue {
        self.transaction_id
    }
    pub fn is_coinbase_transaction(tx: &Transaction) -> bool {
        tx.get_inputs().is_empty()
    }
    pub fn verify_scripts(
        prev_transaction: &Transaction,
        unlocking_script: &[u8],
        locking_script: &[u8],
    ) -> bool {
        let (signature, public_key) = unlocking_script.split_at(64);

        // verify public key
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let result: [u8; 32] = hasher.finalize().into();

        // check if the hash of public key is the same as the locking script
        if result.to_vec() != *locking_script {
            return false;
        }

        // verify signature
        let msg = Message::from_digest(*prev_transaction.sha256());
        let signature = Signature::from_compact(signature).unwrap();

        // deserialize public key
        let public_key = match PublicKey::from_slice(public_key) {
            Ok(public_key) => public_key,
            Err(e) => {
                println!("Error: {}", e);
                return false;
            }
        };

        if signature.verify(&msg, &public_key).is_err() {
            return false;
        }

        true
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Transaction ID: {}", self.transaction_id)?;
        writeln!(f, "Transaction Fee: {}", self.transaction_fee)?;
        writeln!(f, "Inputs:")?;
        for input in &self.inputs {
            writeln!(
                f,
                "\tPrevious Transaction Hash: {}",
                input.prev_transaction_hash
            )?;
            writeln!(f, "\tPrevious Block Index: {}", input.prev_block_index)?;
            writeln!(f, "\tPrevious Output Index: {}", input.prev_output_index)?;
            writeln!(
                f,
                "\tLength of Unlock Script: {}",
                input.length_of_unlock_script
            )?;
            writeln!(
                f,
                "\tUnlock Script: {}",
                bytes_vec_to_hex_string(&input.unlock_script)
            )?;
        }
        writeln!(f, "Outputs:")?;
        for output in &self.outputs {
            writeln!(f, "\tAmount: {}", output.amount)?;
            writeln!(
                f,
                "\tLength of Locking Script: {}",
                output.length_of_locking_script
            )?;
            writeln!(
                f,
                "\tLocking Script: {}",
                bytes_vec_to_hex_string(&output.locking_script)
            )?;
        }
        writeln!(f, "Additional Data: {:?}", self.additional_data)?;
        Ok(())
    }
}

impl Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let hash = self.sha256();
        hash.iter().for_each(|byte| state.write_u8(*byte));
    }
}

/// verify the unlocking script
/// one must provide the unlocking script and the corresponding locking script.
/// previous transaction need to be provided as the transaction hash is needed.

/// Represents an input for a transaction.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct Input {
    prev_transaction_hash: HashValue, // The hash of the previous transaction.
    prev_block_index: usize,          // The index of the previous block.
    prev_output_index: usize,         // The index of the output in the previous transaction.
    length_of_unlock_script: usize,   // The length of the unlock script.
    unlock_script: Vec<u8>,           // The unlock script: a signature and a public key.
}

impl Input {
    /// Creates a new input for a transaction.
    ///
    /// # Arguments
    ///
    /// * `transaction_hash` - The hash of the previous transaction.
    /// * `previous_block_index` - The index of the previous block.
    /// * `output_index` - The index of the output in the previous transaction.
    /// * `unlock_script` - The unlock script. This is sender's signature and sender's public key.
    pub fn new(
        transaction_hash: HashValue,
        previous_block_index: usize,
        output_index: usize,
        unlock_script: Vec<u8>,
    ) -> Self {
        let length_of_unlock_script = unlock_script.len();
        Self {
            prev_transaction_hash: transaction_hash,
            prev_block_index: previous_block_index,
            prev_output_index: output_index,
            length_of_unlock_script,
            unlock_script,
        }
    }

    pub fn get_prev_tx_hash(&self) -> HashValue {
        self.prev_transaction_hash
    }
    pub fn get_prev_block_index(&self) -> usize {
        self.prev_block_index
    }
    pub fn get_unlock_script(&self) -> &Vec<u8> {
        &self.unlock_script
    }
    pub fn get_prev_output_index(&self) -> usize {
        self.prev_output_index
    }

    /// Generates the unlock script for the input.
    /// an unlock script is a signature of previous transaction and a public key of sender,
    /// here is separated with a 31 bytes long 0s vector `[0u8;31]`.
    /// # Arguments
    /// * `previous_transaction` - The previous transaction.
    /// * `private_key` - The private key of the sender.
    /// * `public_key` - The public key of the sender.
    pub fn generate_unlock_script(
        previous_transaction: &Transaction,
        private_key: SecretKey,
        public_key: PublicKey,
    ) -> Vec<u8> {
        let msg = Message::from_digest(*previous_transaction.sha256());
        let signature = private_key.sign_ecdsa(msg);

        [
            signature.serialize_compact().to_vec(), // signature
            public_key.serialize().to_vec(),        // public key
        ]
        .concat()
    }
}

/// Represents an output for a transaction.
#[derive(Debug, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct Output {
    amount: Decimal,                 // The amount of the output.
    length_of_locking_script: usize, // The length of the locking script.
    locking_script: Vec<u8>,         // The locking script: a public key hash.
}

impl Output {
    /// Creates a new output for a transaction.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount of the output.
    /// * `locking_script` - The locking script. This is a public key hash.
    pub fn new(amount: Decimal, locking_script: Vec<u8>) -> Self {
        let length_of_locking_script = locking_script.len();
        Self {
            amount,
            length_of_locking_script,
            locking_script,
        }
    }

    pub fn get_amount(&self) -> Decimal {
        self.amount
    }
    pub fn get_locking_script(&self) -> &Vec<u8> {
        &self.locking_script
    }

    /// generates the locking script for the output.
    /// a locking script is a hash of public key of receiver.
    pub fn generate_locking_script(public_key: PublicKey) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(public_key.serialize());
        let result: [u8; 32] = hasher.finalize().into();
        result.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use secp256k1::generate_keypair;

    fn create_default_transaction() -> Transaction {
        let mut transaction = Transaction::new(
            vec![Input::new(HashValue::new([0u8; 32]), 0, 0, vec![0u8; 32])],
            vec![Output::new(dec!(0.0), vec![0u8; 32])],
            HashValue::new([0u8; 32]),
            dec!(0.0),
            None,
        );

        transaction.update_digest();

        transaction
    }

    #[test]
    fn test_hasher() {
        let transaction = create_default_transaction();
        // println!("{}", serde_json::to_string(&transaction).unwrap());

        let hash = transaction.sha256(); //hash once
        println!("{}", hash);
        let hash = transaction.sha256().sha256(); //hash twice
        println!("{}", hash);
    }

    #[test]
    fn test_scripts() {
        let transaction = create_default_transaction();

        let (private_key, public_key) = generate_keypair(&mut rand::thread_rng());
        let unlocking_script = Input::generate_unlock_script(&transaction, private_key, public_key);
        let locking_script = Output::generate_locking_script(public_key);
        let res = Transaction::verify_scripts(&transaction, &unlocking_script, &locking_script);
        assert!(res);
    }
}
