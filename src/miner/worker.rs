use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::network::message;
use crate::types::block::Block;
use crate::network::server::Handle as ServerHandle;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::Blockchain;
use crate::types::hash::Hashable;
#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>, 
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        _blockchain: &Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain: Arc::clone(_blockchain), 
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _block = self.finished_block_chan.recv().expect("Receive finished block error");
            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            // update the blockchain 
            //let mut blockchain = self.blockchain.lock().unwrap();
            //blockchain.insert(&_block);
            debug!("Insert a mined block {:?} to blockchain", _block.hash());
            //broadcast 
            let mut new_blocks = Vec::new();
            new_blocks.push(_block.hash());
            self.server.broadcast(message::Message::NewBlockHashes(new_blocks));
            debug!("Broadcast a new block hash {:?} to peers", _block.hash());
        }
    }
}
