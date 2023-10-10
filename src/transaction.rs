use std::collections::HashMap;
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};


// #[derive(Debug,Ord, PartialOrd, Eq, PartialEq,Serialize, Deserialize)]
// pub struct Transaction {
//     sender: HashString,
//     receiver: HashString,
//     amount: f64,
//     timestamp: String,
//     hash: HashString,
//     signature: HashString,
// }
// #[derive(Debug,Ord, PartialOrd, Eq, PartialEq,Serialize, Deserialize)]
// pub struct TransactionPool {
//     pool: Box<HashMap<HashString, Transaction>>,
// }