use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

pub type HashValue = Bytes<32>;

impl HashValue {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

pub type Signature = Bytes<65>;

//to store the hash value on stack, facilitate compute process
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(try_from = "String", into = "String")]
pub struct Bytes<const T: usize>([u8; T]);

impl<const T: usize> TryFrom<String> for Bytes<T> {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let val = if let Some(val) = value.strip_prefix("0x") {
            val
        } else {
            &value
        };

        Ok(hex::decode(val).map(|bytes| {
            let mut arr = [0u8; T];
            arr.copy_from_slice(&bytes);
            Bytes(arr)
        })?)
    }
}

impl<const T: usize> From<Bytes<T>> for String {
    fn from(value: Bytes<T>) -> Self {
        String::from("0x") + &hex::encode(value.0)
    }
}

impl<const T: usize> Bytes<T> {
    fn fmt_as_hex(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;

        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl<const T: usize> fmt::Debug for Bytes<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
    }
}

impl<const T: usize> fmt::Display for Bytes<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
    }
}

impl<const T: usize> Deref for Bytes<T> {
    type Target = [u8; T];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const T: usize> AsRef<[u8]> for Bytes<T> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{Bytes, HashValue};
    use sha2::{Digest, Sha256};

    #[test]
    fn bytes_into() {
        let hash = Sha256::digest(b"hello world").into();
        let hash = HashValue::from(Bytes(hash));
        assert_eq!(
            hash.to_string(),
            "0xb94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
}
