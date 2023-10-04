use crate::types::block::{self, Block};
use crate::types::hash::{Hashable, H256};
use std::collections::HashMap;
pub struct BlockWithHeight {
    pub block: Block,
    ///height is useful when handling uncle blocks
    pub height: u32,
}

pub struct Blockchain {
    /// we save all blocks in a hashmap, key is the hash of the block, value is (block, height)
    pub blocks: HashMap<H256, BlockWithHeight>,
    pub tail_block: H256,
    pub height: u32,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        // unimplemented!()
        let mut blocks = HashMap::new();
        let genesis_block = Block::genesis();
        let genesis_hash = genesis_block.hash();
        blocks.insert(
            genesis_hash,
            BlockWithHeight {
                block: genesis_block,
                height: 0,
            },
        );
        Blockchain {
            blocks,
            tail_block: genesis_hash,
            height: 0,
        }
    }

    /// Insert a block into blockchain, the block.parent must in the blockchain
    pub fn insert(&mut self, block: &Block) {
        //unimplemented!()
        //check if the block is already in the blocks
        if self.blocks.contains_key(&block.hash()) {
            return;
        }
        if block.get_parent() == self.tail_block {
            let block_hash = block.hash();
            self.blocks.insert(
                block_hash,
                BlockWithHeight {
                    block: block.clone(),
                    height: self.height + 1,
                },
            );
            self.tail_block = block_hash;
            self.height += 1;
        } else {
            //handle a fork
            let block_hash = block.hash();
            let block_parent = block.get_parent();
            let block_parent_height = self.blocks.get(&block_parent).unwrap().height;
            let block_height = block_parent_height + 1;
            if block_height > self.height {
                // the fork is longer than the current chain
                //update the tail block and height
                self.tail_block = block_hash;
                self.height = block_height;
            }
            self.blocks.insert(
                block_hash,
                BlockWithHeight {
                    block: block.clone(),
                    height: block_height,
                },
            );
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        //unimplemented!()
        self.tail_block
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        // unimplemented!()
        let mut blocks = vec![];
        let mut block_hash = self.tail_block;
        while block_hash != H256::default() {
            blocks.push(block_hash);
            block_hash = self.blocks.get(&block_hash).unwrap().block.get_parent();
        }
        // reverse the blocks
        blocks.reverse();
        blocks
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());
    }
    /// Test 20 blocks insert into blockchain, and read the longest chain
    #[test]
    fn insert_twenty() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let mut block = generate_random_block(&genesis_hash);
        for _ in 0..20 {
            blockchain.insert(&block);
            assert_eq!(blockchain.tip(), block.hash());
            block = generate_random_block(&block.hash());
        }
        assert_eq!(blockchain.tip(), block.get_parent());
        let blocks = blockchain.all_blocks_in_longest_chain();
        assert_eq!(blocks.len(), 21);
    }
    /// Test forks
    /// 1 -- 2 -- 4
    ///  \- 3 -- 5 -- 6
    #[test]
    fn insert_fork() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block1 = generate_random_block(&genesis_hash);
        let block2 = generate_random_block(&block1.hash());
        let block3 = generate_random_block(&block1.hash());
        let block4 = generate_random_block(&block2.hash());
        let block5 = generate_random_block(&block3.hash());
        let block6 = generate_random_block(&block5.hash());
        blockchain.insert(&block1);
        blockchain.insert(&block2);
        blockchain.insert(&block3);
        blockchain.insert(&block4);
        assert_eq!(blockchain.tip(), block4.hash());
        blockchain.insert(&block5);
        blockchain.insert(&block6);
        assert_eq!(blockchain.tip(), block6.hash());
        let blocks = blockchain.all_blocks_in_longest_chain();
        assert_eq!(blocks.len(), 5);
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
