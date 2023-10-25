#[macro_use] extern crate hex_literal;
mod types; 
use ring::signature::{Ed25519KeyPair, KeyPair};
use types::hash::{Hashable, H256};
use types::key_pair::{self, PublicKey};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path; 
use crate::types::transaction; 
pub struct Wallet{
    key: Ed25519KeyPair,
    my_utxo: Vec<(H256, usize, transaction::Output)>,
    pkh: H256, 
    rpc_addr: String,
}

impl Wallet{
    pub fn new(key_path: &str, rpc_addr: &str) -> Self {
        let root_path = Path::new(key_path);
        let mut key_file = fs::File::open(root_path).unwrap();
        let mut key = Vec::new();
        key_file.read_to_end(&mut key).unwrap();
        let key = Ed25519KeyPair::from_pkcs8(&key).unwrap();
        let pkh= key.public_key().as_ref().to_vec().hash();
        Wallet{
            key: key,
            my_utxo: Vec::new(),
            rpc_addr: rpc_addr.to_string(),
            pkh: pkh,
        }
    }
   
    pub fn update_utxo(){
        // call rpc to get utxo 
        // the rpc path is rpc_addr + "/mempool/query_utxo?by_pk?pkh=" + pkh 
        // the rpc return a json array of utxo
        // json is a list, each element is a tuple of (tx_hash, index, output)
        // for each element, add it to my_utxo
        
        //first construct url 
        
    }
}


fn main() {
    println!("This is wallet"); 
}
