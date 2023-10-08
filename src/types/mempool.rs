use super::block::{Body, Block};
use super::merkle::MerkleTree;
/* 
What mempool do? 
1. add singed tx to the mempool, check it before add it to the mempool
2. propose a  block body, and send it to the miner

How? 
1. maintain a list of pending tx 
2. check the tx before add it to the mempool 
    1. check if the tx is valid 
    2. check if the tx is double spent
        1. check the input is not in the blockchain's utxo (no double spent on chain)
        2. check the input is not in the mempool's utxo (no double spent in mempool)
    3. check if the tx is already in the mempool
3. order the tx in the mempool by fee (omit the dependency in the mempool)
4. set a max size for the mempool, if the mempool is full, remove the tx with lowest fee
5. when a new block is mined, remove the tx in the block from the mempool
    1. this means the mempool should hold a channel from the miner, and the miner will send the block to the mempool
6. when a new block is received, remove the tx in the block from the mempool
    1. this means the mempool should hold a channel from the network, and the network will send the block to the mempool    

Interface 
1. add tx: for client add a new tx to the mempool 
2. generate block: propose a valid block body and corresponding merkle root for the tx body 
3. receive block: receive a finalized block from the blockchain when a new block is added to the blockchain, and the finalized block changed 
    1. remove the tx in the block from the mempool
    2. update utxo, remove used utxo and add new utxo 
    3. every time update the utxo, check every tx in the mempool, if the tx is valid, add it to the mempool
4. revoke block: receive a block that is revoked from the blockchain, and the finalized block changed 
    1. remove the outputs from the revoked block from the utxo, add the inputs from the revoked block to the utxo 
    2. check everty tx in the mempool 
*/
use super::transaction::{Transaction, SignedTransaction, Input, Output};
use super::hash::{H256, Hashable};
use std::collections::HashMap;
pub struct Mempool {
    // k: tx_hash, v: signed_tx
    pub txs: Vec<SignedTransaction>,
    // utxo, this utxo is aligned with the finialized block and current mempool txs 
    pub utxo: HashMap<(H256, usize), UTXO>,
}

pub struct UTXO{
    pub output: Output,
    /// if the state = false, means the utxo is not used from the mempool pending tx, if the state = true, means the utxo is used from the mempool pending tx
    pub used_in_mempool: bool,
}
impl Mempool {
    pub fn new() -> Self {
        Mempool {
            txs: Vec::new(),
            utxo: HashMap::new(),
        }
    }
    /// get the corespoinding output for the tx from utxo 
    pub fn get_utxo(&self, tx: &SignedTransaction) -> Result< Vec<Output>, String> {
        let mut outputs = Vec::new();
        for input in &tx.transaction.inputs {
            let output = self.utxo.get(&(input.source_tx_hash, input.index));
            match output {
                Some(output) => {
                    if output.used_in_mempool {
                        return Err("Double spent in mempool".to_string());
                    }
                    outputs.push(output.output.clone());
                }
                None => return Err("No input not in utxo".to_string()),
            }
        }
        Ok(outputs)
    }
    /// add a tx to the mempool 
    pub fn add_tx(&mut self, tx: &SignedTransaction) -> Result<(), String> {
        //get utxo for the tx
        let outputs = self.get_utxo(tx)?;
        // check if the tx is valid
        if tx.verify(&outputs) < 0 {
            return Err("Invalid tx".to_string());
        }
        // update utxo 
        for (_, input) in tx.transaction.inputs.iter().enumerate() {
            // set the used_in_mempool to true
            let mut output = self.utxo.get_mut(&(input.source_tx_hash, input.index)).unwrap();
            output.used_in_mempool = true;
        } 
        //
        Ok(())
    }
    /// generate a block body for the miner, return blody, merkle root and total fee 
    pub fn propose_block_body(&self) -> (Body, H256, u32){
        let body = Body{
            tx_count: self.txs.len(),
            txs: self.txs.clone(),
        };
        let mut total_fee = 0;
        for tx in &self.txs {
            total_fee += tx.fee;
        }
        let merkle_tree = MerkleTree::new(&self.txs);
        let merkle_root = merkle_tree.root();
        (body, merkle_root, total_fee)
    }
    /// receive a finalized block from the blockchain, update utxo and txs 
    pub fn receive_finalized_block(&mut self, block: &Block) -> Result<(), String> {
        // remove the tx in the block from the mempool
        for tx in &block.body.txs {
            self.txs.retain(|x| x.get_tx_hash() != tx.get_tx_hash());
        }
        // update utxo, remove used utxo and add new utxo
        //add new utxo 
        for (_, tx) in block.body.txs.iter().enumerate() {
            for (index, output) in tx.transaction.outputs.iter().enumerate() {
                let key = (tx.get_tx_hash(), index);
                let utxo = UTXO{
                    output: output.clone(),
                    used_in_mempool: false,
                };
                self.utxo.insert(key, utxo);
            }
        }
        // remove used utxo
        for tx in &block.body.txs {
            for input in &tx.transaction.inputs {
                self.utxo.remove(&(input.source_tx_hash, input.index));
            }
        }
        self.check_mempool();
        Ok(())
    }
    /// check every tx in the mempool, if the tx is not valid, remove it, and set the utxo used_in_mempool flag to false
    pub fn check_mempool(&mut self)  {
        // First set all invalid tx's used utxo's flag (used_in_mempool) to false
        // Second remove all these invalid tx from the txs 
         let mut invalid_txs = Vec::new();
            for tx in &self.txs {
                let outputs = self.get_utxo(tx);
                match outputs {
                    Ok(outputs) => {
                        if tx.verify(&outputs) < 0 {
                            invalid_txs.push(tx.clone());
                        }
                    }
                    Err(_) => {
                        invalid_txs.push(tx.clone());
                    }
                }
            }
        // remove invalid tx from the txs
        for tx in invalid_txs {
            self.txs.retain(|x| x.get_tx_hash() != tx.get_tx_hash());
        }
    }
    /// receive a revoked block from blockchain, undo all the tx in the blockchain, the block must be valid, the revoke must take from the tip of the blockchain, to the common prefix of the blockchain 
    pub fn receive_revoked_block(&mut self, block: &Block) -> Result<(), String> {
        // undo this block 
        // remove the outputs from the revoked block from the utxo, add the inputs from the revoked block to the utxo
        let mut outputs = Vec::new();
        for tx in &block.body.txs {
            for (index, output) in tx.transaction.outputs.iter().enumerate() {
                let key = (tx.get_tx_hash(), index);
                outputs.push((key, output.clone()));
            }
        }
        // add those ouputs to the utxo
        for (key, output) in outputs {
            let utxo = UTXO{
                output: output,
                used_in_mempool: false,
            };
            self.utxo.insert(key, utxo);
        }
        // remove those inputs from the utxo
        let mut inputs = Vec::new();
        for tx in &block.body.txs {
            for input in &tx.transaction.inputs {
                inputs.push((input.source_tx_hash, input.index));
            }
        }
        // get the inputs' corespoinding outputs from the blockchain 
        // this should call a new function 
        // todo 
        // call check_mempool 

        Ok(())
    }
}