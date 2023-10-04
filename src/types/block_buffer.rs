use std::collections::HashMap;
use std::sync::{Arc, Mutex};


use super::block::Block;
use crate::Blockchain;
use super::hash::{H256, Hashable};

pub struct BlockBuffer {
    buffer: HashMap<H256, Block>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl BlockBuffer{
    pub fn new(_blockchain: &Arc<Mutex<Blockchain>>) ->  BlockBuffer{

        BlockBuffer{
            buffer: HashMap::new(),
            blockchain: Arc::clone(_blockchain),
        }

    }
    /// if the parent of the block is in the blockchain, add the block to the blockchain, or add to the buffer, return true if the block is added to the blockchain
    pub fn send_block(&mut self, _block: Block) -> bool{
        // check if the parent of the block is in the blockchain
        
        let parent_hash = _block.header.parent;
        let mut if_in= false; 
        {
            if_in = self.blockchain.lock().unwrap().blocks.contains_key(&parent_hash);
        }
        if !if_in{
            self.buffer.insert(_block.hash(), _block);
            return false ;
        }
        self.push_block(_block);
        true
    }
    fn push_block(&mut self, _block: Block){
        let mut blockchain_unlocked = self.blockchain.lock().unwrap();
        blockchain_unlocked.insert(&_block);
        // check buffer, push all pushable blocks from buffer to chain 
        loop {
            let mut to_remove = Vec::new();
            let mut added = false;
            // range buffer
            for (hash, block) in &self.buffer {
                if blockchain_unlocked.blocks.contains_key(&block.header.parent) {
                    blockchain_unlocked.insert(block);
                    to_remove.push(hash.clone());
                    added = true;
                }
            }
            for id in to_remove {
                self.buffer.remove(&id);
            }
            if !added{
                break; 
            }
        }

     
    }

}

