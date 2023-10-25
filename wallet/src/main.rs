#[macro_use] extern crate hex_literal;
mod types; 
use ring::signature::{Ed25519KeyPair, KeyPair};
use types::hash::{Hashable, H256};
use types::key_pair::{self, PublicKey};
use std::fs;
use std::io::{self, Read, Write, BufReader, BufRead};
use std::path::Path; 
use crate::types::transaction; 
use ureq; 
use serde_json; 
use types::transaction::{Output, Transaction};

use clap::{Arg, App, SubCommand};
pub struct Wallet{
    key: Ed25519KeyPair,
    my_utxo: Vec<(H256, usize, Output)>,
    pkh: H256, 
    rpc_addr: String,
    //neighbors pkh 
    neighbors: Vec<H256>, 
}

impl Wallet{
    pub fn new(key_path: &str, rpc_addr: &str) -> Self {
        let root_path = Path::new(key_path);
        let mut key_file = fs::File::open(root_path).unwrap();
        let mut key = Vec::new();
        key_file.read_to_end(&mut key).unwrap();
        let key = Ed25519KeyPair::from_pkcs8(&key).unwrap();
        let pkh= key.public_key().as_ref().to_vec().hash();
        let n : Vec<H256> = Vec::new();
        Wallet{
            key: key,
            my_utxo: Vec::new(),
            rpc_addr: rpc_addr.to_string(),
            pkh: pkh,
            neighbors:n,
        }
    }
   
    pub fn update_utxo(& mut self){
        // call rpc to get utxo 
        // the rpc path is rpc_addr + "/mempool/query_utxo?by_pk?pkh=" + pkh 
        // the rpc return a json array of utxo
        // json is a list, each element is a tuple of (tx_hash, index, output)
        // for each element, add it to my_utxo
        
        //first construct url 
        let pkh_str = self.pkh.to_string();
        let url = self.rpc_addr.clone() + "/mempool/query_utxo_by_pk?pkh=" + &pkh_str;
        let response = ureq::get(&url).call().unwrap_or_else(|e| panic!("request error {}", e));

        let resp = response.into_string().unwrap();
        let utxos: Vec<(H256, usize, Output)>  = serde_json::from_str(&resp).unwrap(); 
        for utxo in utxos{
            self.my_utxo.push(utxo);
        }
        //get the total amount of all utxos 
        let mut total_amount = 0;
        for utxo in &self.my_utxo{
            total_amount += utxo.2.value;
        }
        println!("Total amount: {}", total_amount);
        //println!("utxos: {:?}", utxos);
        
    }

    pub fn load_neighbors(&mut self, path: &str){
        // load neighbors from file 
        // the file is a json array of pkh 
        // for each element, add it to neighbors 
        let root_path = Path::new(path);
        let pkh_file = fs::File::open(root_path).unwrap();
        // read pkh_file line by line,each line is a pkh
        let reader = BufReader::new(pkh_file);
        for line in reader.lines() {
            let pkh = line.unwrap().parse::<H256>().unwrap();
            self.neighbors.push(pkh);
        }
        println!("neighbors: {:?}", self.neighbors);
    }
}




fn main() {
    let matches = App::new("RBTC Wallet")
        .version("0.1.0")
        .author("PlasticBug")
        .about("Check you account and tranfer your RBTC!")
        .arg(Arg::with_name("key")
            .short("k")
            .long("key")
            .value_name("FILE")
            .help("Sets the key file")
            .takes_value(true)
            .default_value("../keys/alice.key"))
        .arg(Arg::with_name("address")
            .short("a")
            .long("address")
            .value_name("ADDRESS")
            .help("Sets the server address")
            .takes_value(true)
            .default_value("http://127.0.0.1:7000"))
        .arg(Arg::with_name("neighbors")
            .short("n")
            .long("neighbors")
            .value_name("FILE")
            .help("Sets the neighbors file")
            .takes_value(true)
            .default_value("../pks.txt"))
            .subcommand(SubCommand::with_name("show_utxo_detail")
            .about("Shows UTXO details"))
        .subcommand(SubCommand::with_name("transfer")
            .about("Transfers x RBTC to pkh")
            .arg(Arg::with_name("to")   
                .short("t")
                .long("to")
                .value_name("PkHash")
                .help("Sets the receiver's address")
                .takes_value(true))
            .arg(Arg::with_name("amount")
                .short("a")
                .long("amount")
                .value_name("AMOUNT")
                .help("Sets the amount to transfer")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("transfer_by_id")
            .about("Transfers RBTC to an neighbor with index i")
            .arg(Arg::with_name("to")   
                .short("t")
                .long("to")
                .value_name("NUMBER")
                .help("Sets the receiver's address")
                .takes_value(true))
            .arg(Arg::with_name("amount")
                .short("a")
                .long("amount")
                .value_name("AMOUNT")
                .help("Sets the amount to transfer")
                .takes_value(true)))
        .get_matches();

    // Gets a value for key and address if supplied by user, or defaults
    let key_file = matches.value_of("key").unwrap();
    let address = matches.value_of("address").unwrap();
    let neighbors_file = matches.value_of("neighbors").unwrap();

    //init wallet 
    println!("Open Wallet: "); 
    let mut wallet = Wallet::new(key_file, address);
    println!("My public key hash is {:?}", wallet.pkh);
    wallet.update_utxo();
    println!("UTXO count: {:?}", wallet.my_utxo.len());
    wallet.load_neighbors(neighbors_file);
}
