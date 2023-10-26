use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::block::Block;
use super::hash::{Hashable, H256};
use crate::types::mempool::Mempool;
use crate::{blockchain::K, Blockchain};
use log::{debug, info};
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
    pub fn send_block(
        &mut self,
        _block: Block,
        blockchain: &Arc<Mutex<Blockchain>>,
        mempool: &Arc<Mutex<Mempool>>,
    ) -> bool {
        // check if the parent of the block is in the blockchain

        let parent_hash = _block.header.parent;
        let mut if_in = false;
        let mut blockchain_unlocked = blockchain.lock().unwrap();
        if_in = blockchain_unlocked.blocks.contains_key(&parent_hash);
        if !if_in {
            // for orphan blocks, just insert into blocks
            self.buffer.insert(_block.hash(), _block);
            return false;
        }
        //This thread will hold the blockchain lock until the block is added to the blockchain
        self.push_block(_block, &mut blockchain_unlocked, mempool);
        true
    }
    /// push the block with parent in buffer to chain, is function is like a closure which can only be called by fn send_block
    fn push_block(
        &mut self,
        _block: Block,
        blockchain_unlocked: &mut Blockchain,
        mempool: &Arc<Mutex<Mempool>>,
    ) {
        //just throw invalid PoW block with parents, for currently invalid orphan PoW block, we save them in buffer
        if _block.hash() > blockchain_unlocked.get_difficulty() {
            return;
        }
        let mut unlocked_mempool = mempool.lock().unwrap();
        blockchain_insert_with_mempool_atomic(
            _block,
            blockchain_unlocked,
            &mut unlocked_mempool,
        );
        // check buffer, push all pushable blocks from buffer to chain
        loop {
            let mut to_remove = Vec::new();
            let mut added = false;
            let mut current_buffer = self.buffer.clone();
            // range buffer
            for (hash, block) in current_buffer {
                if blockchain_unlocked
                    .blocks
                    .contains_key(&block.header.parent)
                {
                    if block.hash() <= blockchain_unlocked.get_difficulty() {
                        blockchain_insert_with_mempool_atomic(
                            block.clone(),
                            blockchain_unlocked,
                            &mut unlocked_mempool,
                        );
                        to_remove.push(hash);
                        added = true;
                    } else {
                        //once the invalid pow orphan block find its mom, remove invalid PoW block
                        to_remove.push(hash);
                    }
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
pub fn blockchain_insert_with_mempool_atomic(
    _block: Block,
    blockchain_unlocked: &mut Blockchain,
    unlocked_mempool: &mut Mempool,
) {
    let (not_fork, new_finalized_block_hash) = blockchain_unlocked.insert(&_block);
    // this means the new finalized block is not the child of the current fn blk, so we need to rebuild utxo and flush the mempool
    if !not_fork {
        //rebuild utxo and flush the mempool
        // get the new block from genesis to the fn block
        let new_blks = blockchain_unlocked.get_all_blocks_from_genesis_to_finialized();
        // rebuild utxo
        unlocked_mempool.rebuild_utxo(&new_blks);
    } else {
        // just update utxo and mempool
        // check if the new finalized the block is higher
        if blockchain_unlocked.height > K
            && blockchain_unlocked.height - K > unlocked_mempool.synced_block_height
        {
            // update the mempool using the new finalized block
            let new_fb = blockchain_unlocked
                .blocks
                .get(&new_finalized_block_hash)
                .unwrap();
            //info!("Update mempool using new finalized block {:?}", new_fb.block.hash());
            unlocked_mempool
                .receive_finalized_block(&new_fb.block)
                .unwrap();
        }
    }
}
