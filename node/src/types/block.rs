use serde::{Serialize, Deserialize};
use crate::types::{hash::{H256, Hashable }, transaction::Transaction};
use super::{transaction::{SignedTransaction, Output}, merkle::MerkleTree, key_pair::PublicKey};
use rand::prelude::*;
use super::ico::IcoGenerator;
use super::ico; 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    /// prev block hash 
    pub parent: H256,
    pub difficulty: H256,
    pub merkle_root: H256,
    pub timestamp: std::time::SystemTime,
    pub nonce: u32,
}
impl Hashable for Header {
    fn hash (&self) -> H256 {
        let header_bytes = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &header_bytes).into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body {
    pub tx_count: usize, 
    pub txs: Vec<SignedTransaction>, 
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: Header,
    pub body: Body,
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
    /// genesis will return a static block 
    pub fn genesis() -> Block{
        let body = generate_ico_body();
        let header = generate_genesis_header();
        let mut gb = Block { header: header, body: body }; 
        loop {
            if gb.hash() < gb.header.difficulty {
                break;
            }
            gb.header.nonce += 1;
        }
        gb
    }
    // gen a new block with 0 nounce 
    pub fn new_block_from_txs(parent: &H256, txs: &Vec<SignedTransaction>) -> Block{
        let body = Body {
            tx_count: txs.len(),
            txs: txs.clone(),
        };
        let mut header = generate_random_header(parent);
        header.nonce = 0; 
        let merkle_tree = MerkleTree::new(&body.txs);
        header.merkle_root = merkle_tree.root();
        Block { header: header, body: body }
    }
}
fn generate_genesis_header() -> Header{
    // generate a 256 bits byte list with 16 bits zero, rest 1 
    let mut difficulty = H256::default();
    difficulty.0[0] = 0;
    difficulty.0[1] = 0;
    for i in 2..32 {
        difficulty.0[i] = 255;
    }
    // make a static time stamp 
    use chrono::{TimeZone, Utc};
    let genesis_time = Utc.ymd(2023, 10, 01).and_hms(0,0,0); 
    let timestamp = std::time::SystemTime::from(genesis_time); 
    let merkle_root = H256::default(); 
    Header { parent: H256::default(), difficulty: difficulty, merkle_root: merkle_root, timestamp: timestamp, nonce: 0}

}
pub fn generate_random_header(parent: &H256) -> Header{
    //gen a difficulty with 10 bit zero in the first 256 bits in H256 

    let mut difficulty = H256::default();
    difficulty.0[0] = 0;
    difficulty.0[1] = 0;
    for i in 2..32 {
        difficulty.0[i] = 255;
    }

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


pub fn generate_empty_body() -> Body {
    Body {
        tx_count: 0,
        txs: Vec::new(),
    }
}
// generate a body with init ico utxo, this will be used in genesis block 
pub fn generate_ico_body() -> Body {
    use ring::signature::KeyPair;
    let mut ico = IcoGenerator::load_key(ico::CONFIG_PATH);
    // generate a tx with 0 input and 3 output, each 1000,000 btc
    // construct the output 
    let mut outputs:Vec<Output> = Vec::new();
    for i in 0..3 {
        let pk: PublicKey = ico[i].public_key().as_ref().to_vec();
        let pk_hash = pk.hash();
        let output = Output{pk_hash: pk_hash, value: 1000000};
        outputs.push(output);
    }
    let ico_tx = Transaction{inputs: Vec::new(), outputs: outputs};
    let signed_ico_tx = SignedTransaction{transaction: ico_tx, fee: 0, witnesses: Vec::new()};
    Body{
        tx_count:1, 
        txs: vec![signed_ico_tx],
    }
}

pub fn generate_random_block(parent: &H256) -> Block {
    //unimplemented!()
    let header = generate_random_header(parent);
    let body = generate_empty_body();
    Block {
        header,
        body,
    }
}