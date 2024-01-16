use crate::transaction::Output;
use crate::types::HashValue;
use secp256k1::{generate_keypair, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::Write;

pub struct UTXO {
    pub output: Output,            // the output
    pub prev_block_index: u64,     // the index of the block that contains the previous transaction
    pub prev_tx_output_index: u64, // the index of the output in the transaction's outputs
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
    unspent_tx_outputs: Vec<Output>,
    address: HashValue, // SHA 256 hash of public key
}

impl Wallet {
    /// Creates a new wallet.
    /// use a random number generator to generate a public key and a secret key.
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

    /// read secret key from a file
    /// public key can be generated from the secret key
    /// return a Wallet
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
    /// later the wallet can be recovered from method `Wallet::build_from_private_key_file(path: &str)`
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
        let wallet = Wallet::new();
        if let Err(e) = wallet.save_private_key_to_file("./key.rscnkey") {
            println!("{}", e)
        };
        let wallet_copied =
            Wallet::build_from_private_key_file("./key.rscnkey").unwrap_or_else(|e| {
                println!("{}", e);
                Wallet::new()
            });
        println!("{:?}", wallet);
        println!("{:?}", wallet_copied);
        assert_eq!(wallet, wallet_copied);
    }
}
