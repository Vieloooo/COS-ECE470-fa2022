#[cfg(test)]
#[macro_use]
extern crate hex_literal;

pub mod api;
pub mod blockchain;
pub mod types;
pub mod miner;
pub mod network;
use blockchain::Blockchain;
use clap::clap_app;
use smol::channel;
use log::{error, info};
use api::Server as ApiServer;
use types::mempool;
use std::net;
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use crate::types::block_buffer::BlockBuffer; 
use crate::types::transaction::Output;
use crate::types::block::Block;
use crate::types::hash::{H256, Hashable};
use crate::types::mempool::{UTXO, Mempool};
use crate::types::key_pair; 
fn main() {
    // parse command line arguments
    let matches = clap_app!(Bitcoin =>
     (version: "0.1")
     (about: "Bitcoin client")
     (@arg verbose: -v ... "Increases the verbosity of logging")
     (@arg peer_addr: --p2p [ADDR] default_value("127.0.0.1:6000") "Sets the IP address and the port of the P2P server")
     (@arg api_addr: --api [ADDR] default_value("127.0.0.1:7000") "Sets the IP address and the port of the API server")
     (@arg known_peer: -c --connect ... [PEER] "Sets the peers to connect to at start")
     (@arg p2p_workers: --("p2p-workers") [INT] default_value("4") "Sets the number of worker threads for P2P server")
    )
    .get_matches();

    // init logger
    let verbosity = matches.occurrences_of("verbose") as usize;
    stderrlog::new().verbosity(verbosity).init().unwrap();
    let blockchain = Blockchain::new();
    // init the genesis process 
    let blockchain = Arc::new(Mutex::new(blockchain));
    let block_buffer = Arc::new(Mutex::new(BlockBuffer::new()));
    let mempool = Arc::new(Mutex::new(Mempool::new()));
    // get genesis outputs from the genesis block
    let genesis_block = blockchain.lock().unwrap().get_all_blocks_from_genesis_to_finialized()[0].clone();
    // extract utxos from genesis_block
    let genesis_utxo = &genesis_block.body.txs[0].transaction.outputs;
    // add utxos to mempool
    let genesis_hash = genesis_block.hash();
    let mut i = 0; 
    for utxo in genesis_utxo {
        mempool.lock().unwrap().add_utxo((genesis_hash, i), UTXO {output: utxo.clone(), used_in_mempool: false});
        i= i + 1; 
    }
    // init a new keypair for this block 
    //let mykey = key_pair::random(); 
    // parse p2p server address
    let p2p_addr = matches
        .value_of("peer_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P server address: {}", e);
            process::exit(1);
        });

    // parse api server address
    let api_addr = matches
        .value_of("api_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing API server address: {}", e);
            process::exit(1);
        });

    // create channels between server and worker
    let (msg_tx, msg_rx) = channel::bounded(10000);

    // start the p2p server
    let (server_ctx, server) = network::server::new(p2p_addr, msg_tx).unwrap();
    server_ctx.start().unwrap();

    // start the network msg handle worker
    let p2p_workers = matches
        .value_of("p2p_workers")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P workers: {}", e);
            process::exit(1);
        });
    let worker_ctx = network::worker::Worker::new(
        p2p_workers,
        msg_rx,
        &server,
        &blockchain,
        &block_buffer,
        &mempool,
    );
    // p2p network worker, handle all the network messages and update the blockchain
    worker_ctx.start();

    // start the miner
    // The miner thread will manage the block update and mining process
    let (miner_ctx, miner, finished_block_chan) = miner::new(&blockchain, &mempool);
    let miner_worker_ctx = miner::worker::Worker::new(&server, finished_block_chan, &blockchain, &mempool);
    // miner main process, just mine a new block and propose to mine worker
    miner_ctx.start();
    miner_worker_ctx.start();

    // connect to known peers
    if let Some(known_peers) = matches.values_of("known_peer") {
        let known_peers: Vec<String> = known_peers.map(|x| x.to_owned()).collect();
        let server = server.clone();
        thread::spawn(move || {
            for peer in known_peers {
                loop {
                    let addr = match peer.parse::<net::SocketAddr>() {
                        Ok(x) => x,
                        Err(e) => {
                            error!("Error parsing peer address {}: {}", &peer, e);
                            break;
                        }
                    };
                    match server.connect(addr) {
                        Ok(_) => {
                            info!("Connected to outgoing peer {}", &addr);
                            break;
                        }
                        Err(e) => {
                            error!(
                                "Error connecting to peer {}, retrying in one second: {}",
                                addr, e
                            );
                            thread::sleep(time::Duration::from_millis(1000));
                            continue;
                        }
                    }
                }
            }
        });
    }


    // start the API server

    ApiServer::start(
        api_addr,
        &miner,
        &server,
        &blockchain,
        &mempool, 
    );

    loop {
        std::thread::park();
    }
}
