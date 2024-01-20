use crate::errors::RustyCoinError;
use crate::errors::RustyCoinError::{InvalidInputFee, InvalidOutputIndex};
use crate::transaction::{Input, Output, Transaction};
use crate::types::HashValue;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use secp256k1::{generate_keypair, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct UTXO {
    pub prev_tx: Transaction,     // the output
    pub prev_block_index: usize,  // the index of the block that contains the previous transaction
    pub prev_output_index: usize, // the index of the output in the transaction's outputs
    pub prev_tx_hash: HashValue,  // Hash of previous transaction
}

impl UTXO {
    pub fn new(prev_tx: Transaction, prev_block_index: usize, prev_output_index: usize) -> Self {
        let prev_tx_hash = prev_tx.sha256();

        UTXO {
            prev_tx,
            prev_block_index,
            prev_output_index,
            prev_tx_hash,
        }
    }
}

/// a wallet contains a public key, a secret key, a list of unspent transaction outputs and an address.
/// the address is the hash of the public key.
/// the wallet should have the following functionalities:
/// - create a new wallet
/// - transfer credit to another wallet / other wallets
/// - export the wallet to a file / import the wallet from a file
#[derive(Debug, PartialEq)]
pub struct Wallet {
    public_key: PublicKey,
    secret_key: SecretKey,
    unspent_tx_outputs: Vec<UTXO>,
    address: HashValue, // SHA 256 hash of public key
}

impl Wallet {
    /// Creates a new wallet.
    /// use a random number generator to generate a public key and a secret key.
    ///
    /// the address is the hash of the public key.
    pub fn new() -> Self {
        let (secret_key, public_key) = generate_keypair(&mut rand::thread_rng());
        let address = public_key_to_hash(public_key);
        Wallet {
            public_key,
            secret_key,
            unspent_tx_outputs: Vec::new(),
            address,
        }
    }
    /// transfer credit to another wallet / other wallets.
    /// create a signed transaction and sign it with the secret key.
    /// # Arguments
    /// * `utxos`: a vector of UTXO in your wallet
    /// * `receivers`: a vector of tuple, the tuple is consist of
    ///     - a `amount: Decimal` in rust_decimal, the amount that transfer to target
    ///     - a `address: HashValue`, the address of the receiver (or public key hash of receiver)
    pub fn transfer_credits(
        &self,
        utxos: Vec<UTXO>,
        receivers: Vec<(Decimal, HashValue)>,
        extra_info: Option<Vec<u8>>,
    ) -> Result<Transaction, RustyCoinError> {
        let mut input_fee = dec!(0.0);
        let inputs: Result<Vec<Input>, RustyCoinError> = utxos
            .into_iter()
            .map(|utxo| {
                // create corresponding unlocking script of each input
                let unlocking_script =
                    Input::generate_unlock_script(&utxo.prev_tx, self.secret_key, self.public_key);

                //if the previous transaction do not have enough outputs, return error
                match utxo.prev_tx.get_output_by_index(utxo.prev_output_index) {
                    // sum the input fee
                    Some(output) => input_fee += output.get_amount(),
                    None => return Err(InvalidOutputIndex),
                };

                // create input
                let input = Input::new(
                    utxo.prev_tx_hash,
                    utxo.prev_block_index,
                    utxo.prev_output_index,
                    unlocking_script,
                );

                Ok(input)
            })
            .collect();

        let inputs = inputs?; // Unwrap the Result, return Error if exists

        // create outputs
        let outputs: Vec<Output> = receivers
            .into_iter()
            .map(|(amount, address)| Output::new(amount, address.to_vec()))
            .collect();

        // sum the output fee
        let output_fee: Decimal = outputs.iter().map(|output| output.get_amount()).sum();

        let tx_fee = input_fee - output_fee;

        if tx_fee < dec!(0.0) {
            return Err(InvalidInputFee);
        }

        // create transaction
        let mut tx = Transaction::new(
            inputs,
            outputs,
            HashValue::new([0u8; 32]),
            tx_fee,
            extra_info,
        );

        // update the digest of the transaction
        tx.update_digest();

        Ok(tx)
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn get_address(&self) -> HashValue {
        self.address
    }

    /// read secret key from a file,
    ///
    /// public key can be generated from the secret key,
    ///
    /// return a Wallet.
    pub fn build_from_private_key_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read(path)?;

        let secret_key = SecretKey::from_slice(content.as_slice())?;
        let public_key = PublicKey::from_secret_key(&Secp256k1::new(), &secret_key);

        Ok(Wallet {
            public_key,
            secret_key,
            unspent_tx_outputs: Vec::new(),
            address: public_key_to_hash(public_key),
        })
    }

    /// export the private key to a file
    ///
    /// later the wallet can be recovered from method
    ///
    /// `Wallet::build_from_private_key_file(path: &str)`
    pub fn save_private_key_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;
        file.write_all(&self.secret_key[..])?;
        Ok(())
    }
}
fn public_key_to_hash(public_key: PublicKey) -> HashValue {
    let mut hasher = Sha256::new();
    hasher.update(public_key.serialize());
    HashValue::new(hasher.finalize().into())
}
impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_export_and_import_from_a_file() {
        const FILE_PATH: &str = "./test_key.rscnkey";
        let wallet = Wallet::new();
        if let Err(e) = wallet.save_private_key_to_file(FILE_PATH) {
            println!("{}", e)
        };
        let wallet_copied = Wallet::build_from_private_key_file(FILE_PATH).unwrap_or_else(|e| {
            println!("{}", e);
            Wallet::new()
        });
        println!("{:?}", wallet);
        println!("{:?}", wallet_copied);
        assert_eq!(wallet, wallet_copied);
        // delete the key file after testing
        fs::remove_file(FILE_PATH).expect("Delete Fail: No such file");
    }

    /// in P2PKH, the address and the locking script is the same thing
    #[test]
    fn test_locking_script() {
        let wallet = Wallet::new();
        let locking_script = Output::generate_locking_script(wallet.get_public_key());
        let address = wallet.get_address().to_vec();
        assert_eq!(locking_script, address);
    }

    #[test]
    fn test_transfer_credit() {
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let prev_tx = Transaction::new(
            vec![Input::new(HashValue::new([0u8; 32]), 0, 0, vec![0u8; 32])],
            vec![Output::new(dec!(1.0), wallet1.address.to_vec())],
            HashValue::new([0u8; 32]),
            dec!(0.0),
            None,
        );
        let tx = wallet1
            .transfer_credits(
                vec![UTXO::new(prev_tx, 0, 0)],
                vec![(dec!(0.5), wallet2.get_address())],
                None,
            )
            .unwrap();

        println!("{:?}", tx);
    }
}
