use serde::{Serialize, Deserialize};
use crate::types::hash::{H256, Hashable };
use super::transaction::SignedTransaction;
use rand::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    /// prev block hash 
    parent: H256,
    difficulty: H256,
    merkle_root: H256,
    timestamp: std::time::SystemTime,
    nonce: u32,
}
impl Hashable for Header {
    fn hash (&self) -> H256 {
        let header_bytes = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &header_bytes).into()
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body {
    tx_count: usize, 
    txs: Vec<SignedTransaction>, 
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: Header,
    body: Body,
}

impl Hashable for Block {
    // hash of the block header 
    fn hash(&self) -> H256 {
        //unimplemented!()
        self.header.hash()
    }
}

impl Block {
    pub fn get_parent(&self) -> H256 {
        //unimplemented!()
        self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        //unimplemented!()
        self.header.difficulty
    }
}
#[cfg(any(test, test_utilities))]
pub fn generate_random_header(parent: &H256) -> Header{
    //gen a difficulty with 10 bit zero in the first 256 bits in H256 

    let mut difficulty = crate::types::hash::generate_random_hash();
    // make the first 8 bits in difficulty zero 
    difficulty.0[0] = 0;
    // nounce should be a random u32 using crate ring 
    let mut rng = rand::thread_rng();
    let nounce = rng.gen::<u32>();
    let timestamp = std::time::SystemTime::now();
    let merkle_root = H256::default();
    Header {
        parent: parent.clone(),
        difficulty,
        merkle_root,
        timestamp,
        nonce: nounce,
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_empty_body() -> Body {
    //unimplemented!()
    Body {
        tx_count: 0,
        txs: Vec::new(),
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    //unimplemented!()
    let header = generate_random_header(parent);
    let body = generate_empty_body();
    Block {
        header,
        body,
    }
}