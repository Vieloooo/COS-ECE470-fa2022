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
use types::transaction::{Output, Transaction, Input, Witness, SignedTransaction};

use clap::{Arg, App, SubCommand};
pub struct Wallet{
    key: Ed25519KeyPair,
    my_utxo: Vec<(H256, usize, Output)>,
    pkh: H256, 
    rpc_addr: String,
    //neighbors pkh 
    neighbors: Vec<H256>, 
    balance: u64, 
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
            balance: 0,
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
        self.balance = total_amount;
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
    pub fn submit_tx(&self, transaction: & transaction::SignedTransaction){
        // submit a signed transaction to rpc 
        // the rpc path is rpc_addr + "/mempool/submit_tx"
        // the rpc return a json object of {status: "ok"} or {status: "error"}
        // first construct url 
        let url = self.rpc_addr.clone() + "/mempool/submit_tx";
        let json = serde_json::to_string(&transaction).unwrap();
        let response = ureq::post(&url).send_string(&json).unwrap();
        let resp = response.into_string().unwrap();
        println!("submit tx response: {}", resp);
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
    // Handle subcommands
    match matches.subcommand() {
        ("show_utxo_detail", Some(_)) => {
            // Show UTXO details
            println!("UTXO details: {:?}", wallet.my_utxo);
        }
        ("transfer", Some(transfer_matches)) => {
            // Transfer
            let to = transfer_matches.value_of("to").unwrap();
            let amount = transfer_matches.value_of("amount").unwrap();
            // covert to to pkh
            let to_pkh = to.parse::<H256>().unwrap();
            // covert amount to u64
            let amount = amount.parse::<u64>().unwrap();
            // check if amount is valid
            if amount > wallet.balance{
                println!("Not enough balance!");
                return;
            }
            // create transaction
            // make output first 
            let output = Output{
                pk_hash: to_pkh,
                value: amount,
            };
            // make input
            // range current utxo, add them to input list until the sum of input is larger than amount
            let mut inputs: Vec<transaction::Input> = Vec::new();
            let mut sum = 0;
            
            for utxo in &wallet.my_utxo{
                let input = Input{
                    source_tx_hash: utxo.0,
                    index: utxo.1,
                };
                inputs.push(input);
                sum += utxo.2.value;
                if sum > amount{
                    break;
                }
            }
            // make a charge output 
            let fee :u64 = 1; 
            let charge_output = Output{
                pk_hash: wallet.pkh,
                value: sum - amount - fee,
            };
            //make transaction
            let tx = Transaction{
                inputs: inputs,
                outputs: vec![output, charge_output],
            };
            // make signature 
            let sig = transaction::sign(&tx, & wallet.key);
            let witness = transaction::Witness{
                pubkey: wallet.key.public_key().as_ref().to_vec(),
                sig: sig.as_ref().to_vec(),
            };
            let mut wits: Vec<Witness> = Vec::new(); 
            for _ in 0..tx.inputs.len(){
                wits.push(witness.clone());
            }
            
            // make signed tx
            let signed_tx = transaction::SignedTransaction{
                transaction: tx,
                fee: 1,
                witnesses: wits,
            };
            
            // send signed tx to rpc
            wallet.submit_tx(&signed_tx);



        }
        ("transfer_by_id", Some(transfer_matches)) => {
            // Transfer
            let to = transfer_matches.value_of("to").unwrap();
            let amount = transfer_matches.value_of("amount").unwrap();
            // covert to to index 
            let to_index = to.parse::<usize>().unwrap();
            // covert amount to u64
            let amount = amount.parse::<u64>().unwrap();
            // check if amount is valid
            if amount >= wallet.balance{
                println!("Not enough balance!");
                return;
            }
            // create transaction
            // make output first 
            let output = Output{
                pk_hash: wallet.neighbors[to_index],
                value: amount,
            };
            
            // make input
            // range current utxo, add them to input list until the sum of input is larger than amount
            let mut inputs: Vec<transaction::Input> = Vec::new();
            let mut sum = 0;
            
            for utxo in &wallet.my_utxo{
                let input = Input{
                    source_tx_hash: utxo.0,
                    index: utxo.1,
                };
                inputs.push(input);
                sum += utxo.2.value;
                if sum > amount{
                    break;
                }
            }
            // make a charge output 
            let fee :u64 = 1; 
            let charge_output = Output{
                pk_hash: wallet.pkh,
                value: sum - amount - fee,
            };
            //make transaction
            let tx = Transaction{
                inputs: inputs,
                outputs: vec![output, charge_output],
            };
            // make signature 
            let sig = transaction::sign(&tx, & wallet.key);
            let witness = transaction::Witness{
                pubkey: wallet.key.public_key().as_ref().to_vec(),
                sig: sig.as_ref().to_vec(),
            };
            let mut wits: Vec<Witness> = Vec::new(); 
            for _ in 0..tx.inputs.len(){
                wits.push(witness.clone());
            }
            
            // make signed tx
            let signed_tx = transaction::SignedTransaction{
                transaction: tx,
                fee: 1,
                witnesses: wits,
            };
            //let res = signed_tx.verify()
            // send signed tx to rpc
            //println!("My tx is {:?}", signed_tx);
            wallet.submit_tx(&signed_tx);
        }
        _ => {
            // No subcommand used
            println!("No subcommand used");
        }
    }

}

