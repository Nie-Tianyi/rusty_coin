//! Inplementation of a blockchain

pub mod blockchain {
    use std::collections::HashMap;
    use chrono::{DateTime, Local};

    //alias
    type HashString = [u8;32];
    #[derive(Debug)]
    pub struct Block {
        index: u64,
        data: String,
        timestamp: DateTime<Local>,
        prev_hash: HashString,
        hash: HashString,
        // 0 ~ 255, leading 0 of hash value
        difficulty: u8,
        nonce: i64,
    }

    /// The most frequent operation on a blockchain is Query
    /// Therefore, use a HashMap store the Blocks
    /// key is the `index:u64` of a block, value is the `block:Block`
    struct Blockchain {
        blockchain: Box<HashMap<u64, Block>>,
    }
}

#[cfg(test)]
mod tests{
    use sha2::{Sha256, Digest};
    #[test]
    fn test1(){
        let mut hasher = Sha256::new();
        hasher.update("12345");
        let res = hasher.finalize();
        let hash = res[..].to_owned();
        println!("{:?}",hash);
        println!("{}",hash.len());
        println!("Hello World");
    }
}