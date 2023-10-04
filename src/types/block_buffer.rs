use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::block::Block;
use super::hash::{Hashable, H256};
use crate::Blockchain;

#[derive(Clone)]
pub struct BlockBuffer {
    buffer: HashMap<H256, Block>,
}

impl BlockBuffer {
    pub fn new() -> BlockBuffer {
        BlockBuffer {
            buffer: HashMap::new(),
        }
    }
    /// if the parent of the block is in the blockchain, add the block to the blockchain, or add to the buffer, return true if the block is added to the blockchain
    pub fn send_block(&mut self, _block: Block, blockchain: &Arc<Mutex<Blockchain>>) -> bool {
        // check if the parent of the block is in the blockchain

        let parent_hash = _block.header.parent;
        let mut if_in = false;
        let mut blockchain_unlocked = blockchain.lock().unwrap();
        if_in = blockchain_unlocked
            .blocks
            .contains_key(&parent_hash);
        if !if_in {
            self.buffer.insert(_block.hash(), _block);
            return false;
        }
        //This thread will hold the blockchain lock until the block is added to the blockchain
        self.push_block(_block, &mut blockchain_unlocked );
        true
    }
    fn push_block(&mut self, _block: Block, blockchain_unlocked: &mut Blockchain) {
        blockchain_unlocked.insert(&_block);
        // check buffer, push all pushable blocks from buffer to chain
        loop {
            let mut to_remove = Vec::new();
            let mut added = false;
            // range buffer
            for (hash, block) in &self.buffer {
                if blockchain_unlocked
                    .blocks
                    .contains_key(&block.header.parent)
                {
                    blockchain_unlocked.insert(block);
                    to_remove.push(hash.clone());
                    added = true;
                }
            }
            for id in to_remove {
                self.buffer.remove(&id);
            }
            if !added {
                break;
            }
        }
    }
}
