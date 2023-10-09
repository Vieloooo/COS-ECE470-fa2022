use super::block::{Body, Block};
use super::merkle::MerkleTree;
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
    /// a back-door function for test and genesis initaliation
    pub fn add_utxo(&mut self, key: (H256, usize), utxo: UTXO) {
        self.utxo.insert(key, utxo);
    }
    /// get the corespoinding output for the tx from utxo 
    pub fn get_utxo(&self, tx: &SignedTransaction) -> Result< Vec<Output>, String> {
        let mut outputs = Vec::new();
        for input in &tx.transaction.inputs {
            let output = self.utxo.get(&(input.source_tx_hash, input.index));
            match output {
                Some(output) => {
                    if output.used_in_mempool {
                        return Err("Double spent in mempool, A conflict tx has already added into the mempool. ".to_string());
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
        // update utxo, set the utxo used in the x used in the mempool to true
        for (_, input) in tx.transaction.inputs.iter().enumerate() {
            // set the used_in_mempool to true
            let output = self.utxo.get_mut(&(input.source_tx_hash, input.index)).unwrap();
            output.used_in_mempool = true;
        } 
        //add the tx in txs 
        self.txs.push(tx.clone());
        Ok(())
    }
    /// generate a block body for the miner, return blody, merkle root and total fee 
    pub fn propose_block_body(&self) -> (Body, H256, u32){
        //no block size limitation, put all tx in the mempool to next block 
        let body = Body{
            tx_count: self.txs.len(),
            txs: self.txs.clone(),
        };
        // get total tx_fee in this block 
        let mut total_fee = 0;
        for tx in &self.txs {
            total_fee += tx.fee;
        }
        // build a merkle tree for the txs
        let merkle_tree = MerkleTree::new(&self.txs);
        let merkle_root = merkle_tree.root();
        (body, merkle_root, total_fee)
    }
    /// querying UTXO by public key hash 
    pub fn query_utxo(&self, pk_hash: &H256) -> Vec<Output> {
        let mut outputs = Vec::new();
        for (_, utxo) in &self.utxo {
            if utxo.output.pk_hash == *pk_hash {
                outputs.push(utxo.output.clone());
            }
        }
        outputs
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
            // range every output in the tx, add it to the utxo
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
        if self.txs.len() == 0 {
            return ;
        }
        // get all invalid tx in the mempool 
         let mut invalid_txs_hash = Vec::new();
            for tx in &self.txs {
                let outputs = self.get_utxo(tx);
                match outputs {
                    Ok(outputs) => {
                        if tx.verify(&outputs) < 0 {
                            invalid_txs_hash.push(tx.get_tx_hash());
                        }
                    }
                    Err(_) => {
                        invalid_txs_hash.push(tx.get_tx_hash());
                    }
                }
            }
        // remove invalid tx from the txs
        for tx in invalid_txs_hash {
            self.txs.retain(|x| x.get_tx_hash() != tx);
        }
    }
    /// Rebuild the utxo and empty the mempool when fork 
    /// 1. remove all txs from the mempool
    /// 2. from the block height zero to the fork height, add all blocks to the mempool 
    pub fn rebuild_utxo(&mut self, blocks: &Vec<Block>) {
        // remove all txs from the mempool
        self.txs.clear();
        // from the block height zero to the fork height, add all blocks to the mempool 
        for block in blocks {
            _ = self.receive_finalized_block(&block);
        }
    }
    
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::types::key_pair;
    use crate::types::key_pair::PublicKey;
    use crate::types::transaction::*; 
    use ring::signature::KeyPair;
    /// Test utxo add and query
    /// 1. add a utxo to pk a 100 btc 
    /// 2. add a utxo to pk a 50 btc 
    /// 3. add a utxo to pk b 10 btc 
    /// 4. get the pka's utxo, 
    /// 5. get the pkb's utxo,
    #[test]
    fn mempool_utxo_add_query() {
        let mut mempool = Mempool::new();
        let key_a = key_pair::random();
        let key_b = key_pair::random();
        let mut outputs = Vec::new();
        outputs.push(Output{ pk_hash: key_a.public_key().as_ref().to_vec().hash(), value: 100});
        outputs.push(Output{ pk_hash: key_a.public_key().as_ref().to_vec().hash(), value: 50});
        outputs.push(Output{ pk_hash: key_b.public_key().as_ref().to_vec().hash(), value: 10});
        let mut inputs = Vec::new();
        inputs.push(Input{source_tx_hash: H256::rand(), index: 0});
        inputs.push(Input{source_tx_hash: H256::rand(), index: 0});
        inputs.push(Input{source_tx_hash: H256::rand(), index: 0});
        let tx = Transaction{inputs: inputs, outputs: outputs};
        let signed_tx = SignedTransaction{transaction: tx, fee: 10, witnesses: Vec::new()};
        // add utxo to the mempool
        mempool.add_utxo((signed_tx.get_tx_hash(), 0), UTXO{output: signed_tx.transaction.outputs[0].clone(), used_in_mempool: false});
        mempool.add_utxo((signed_tx.get_tx_hash(), 1), UTXO{output: signed_tx.transaction.outputs[1].clone(), used_in_mempool: false});
        mempool.add_utxo((signed_tx.get_tx_hash(), 2), UTXO{output: signed_tx.transaction.outputs[2].clone(), used_in_mempool: false});
        // query utxo 
        let outputs = mempool.query_utxo(&key_a.public_key().as_ref().to_vec().hash());
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].value + outputs[1].value, 150);
        let outputs = mempool.query_utxo(&key_b.public_key().as_ref().to_vec().hash());
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].value, 10);
    }
    /// Test add a valid tx to the mempool
    /// 1. add a utxo1 to pk a 100 btc
    /// 2. add a utxo2 to pk a 50 btc
    /// 3. add a utxo3 to pk b 10 btc
    /// 4.  add a new tx which spend utxo1 and utxo2 and utxo3, and send 150 btc to pk b, and 10 btc to pk a
    /// 5. check if the tx is in the mempool
    /// 6. check if the utxo is used in the mempool
    /// 7. call propose block body, check the result
    /// 8. check the block size, block merkle and block fee
    #[test]
    fn mempool_tx_add_propose() {
        let mut mempool = Mempool::new();
        let key_a = key_pair::random();
        let key_b = key_pair::random();
        let mut outputs = Vec::new();
        outputs.push(Output{ pk_hash: key_a.public_key().as_ref().to_vec().hash(), value: 100});
        outputs.push(Output{ pk_hash: key_a.public_key().as_ref().to_vec().hash(), value: 50});
        outputs.push(Output{ pk_hash: key_b.public_key().as_ref().to_vec().hash(), value: 10});
        let mut inputs = Vec::new();
        inputs.push(Input{source_tx_hash: H256::rand(), index: 0});
        inputs.push(Input{source_tx_hash: H256::rand(), index: 0});
        inputs.push(Input{source_tx_hash: H256::rand(), index: 0});
        let tx = Transaction{inputs: inputs, outputs: outputs};
        let signed_tx = SignedTransaction{transaction: tx, fee: 10, witnesses: Vec::new()};
        // add utxo to the mempool
        mempool.add_utxo((signed_tx.get_tx_hash(), 0), UTXO{output: signed_tx.transaction.outputs[0].clone(), used_in_mempool: false});
        mempool.add_utxo((signed_tx.get_tx_hash(), 1), UTXO{output: signed_tx.transaction.outputs[1].clone(), used_in_mempool: false});
        mempool.add_utxo((signed_tx.get_tx_hash(), 2), UTXO{output: signed_tx.transaction.outputs[2].clone(), used_in_mempool: false});
        // compose a new tx which spend utxo1 and utxo2 and utxo3, and send 150 btc to pk b, and 10 btc to pk a
        let mut outputs = Vec::new();
        outputs.push(Output{ pk_hash: key_b.public_key().as_ref().to_vec().hash(), value: 140});
        outputs.push(Output{ pk_hash: key_a.public_key().as_ref().to_vec().hash(), value: 10});
        let mut inputs = Vec::new();
        inputs.push(Input{source_tx_hash: signed_tx.get_tx_hash(), index: 0});
        inputs.push(Input{source_tx_hash: signed_tx.get_tx_hash(), index: 1});
        inputs.push(Input{source_tx_hash: signed_tx.get_tx_hash(), index: 2});
        let tx = Transaction{inputs: inputs, outputs: outputs};
        let mut witnesses = Vec::new();
        let sig1 =sign(&tx, &key_a);
        let sig2 =sign(&tx, &key_a);
        let sig3 =sign(&tx, &key_b);
        let witness1 = Witness{pubkey: key_a.public_key().as_ref().to_vec() as PublicKey, sig: sig1.as_ref().to_vec()};
        let witness2 = Witness{pubkey: key_a.public_key().as_ref().to_vec() as PublicKey, sig: sig2.as_ref().to_vec()};
        let witness3 = Witness{pubkey: key_b.public_key().as_ref().to_vec() as PublicKey, sig: sig3.as_ref().to_vec()};
        witnesses.push(witness1);
        witnesses.push(witness2);
        witnesses.push(witness3);
        let tx_tobe_add = SignedTransaction{transaction: tx, fee: 10, witnesses:witnesses};
        // add this signed tx, the result must be ok
        let result = mempool.add_tx(&tx_tobe_add);
        assert_eq!(result.is_ok(), true);
        if result.is_err() {
            println!("{}", result.err().unwrap());
        }
        // check if the tx is in the mempool
        assert_eq!(mempool.txs.len(), 1);
        assert_eq!(mempool.txs[0].get_tx_hash(), tx_tobe_add.get_tx_hash());
        // check if the utxo is used in the mempool
        let utxo1 = mempool.utxo.get(&(signed_tx.get_tx_hash(), 0)).unwrap();
        assert_eq!(utxo1.used_in_mempool, true);
        let utxo2 = mempool.utxo.get(&(signed_tx.get_tx_hash(), 1)).unwrap();
        assert_eq!(utxo2.used_in_mempool, true);
        let utxo3 = mempool.utxo.get(&(signed_tx.get_tx_hash(), 2)).unwrap();
        assert_eq!(utxo3.used_in_mempool, true);
        // call propose block body, check the result
        let (body, _, total_fee) = mempool.propose_block_body();
        // check the block size, block merkle and block fee
        assert_eq!(body.tx_count, 1);
        assert_eq!(body.txs[0].get_tx_hash(), tx_tobe_add.get_tx_hash());
        //assert_eq!(merkle_root, body.merkle_root);
        assert_eq!(total_fee, 10);

    }
    /// Test then mempool receive a finalized block from the blockchain, update utxo and txs
    /// 1. add a utxo1 to pk a 100 btc
    /// 2. add a utxo2 to pk a 50 btc 
    /// 3. add a utxo3 to pk b 10 btc 
    /// 4. 
    /// 5. build a block which contain a tx which spend utxo1 and utxo3, and send 90 btc to pk b, and 10 btc to pk a, 10 to fee
    /// 5. add the block to the mempool by call receive finalized block
    /// 6. check if the utxo are removed from the mempool
    #[test]
    fn mempool_receive_finalized_block()  {
        let mut mempool = Mempool::new();
        let key_a = key_pair::random();
        let key_b = key_pair::random();
        let mut outputs = Vec::new();
        outputs.push(Output{ pk_hash: key_a.public_key().as_ref().to_vec().hash(), value: 100});
        outputs.push(Output{ pk_hash: key_a.public_key().as_ref().to_vec().hash(), value: 50});
        outputs.push(Output{ pk_hash: key_b.public_key().as_ref().to_vec().hash(), value: 10});
        let mut inputs = Vec::new();
        inputs.push(Input{source_tx_hash: H256::rand(), index: 0});
        inputs.push(Input{source_tx_hash: H256::rand(), index: 0});
        inputs.push(Input{source_tx_hash: H256::rand(), index: 0});
        let tx = Transaction{inputs: inputs, outputs: outputs};
        let signed_tx = SignedTransaction{transaction: tx, fee: 10, witnesses: Vec::new()};
        // add utxo to the mempool
        mempool.add_utxo((signed_tx.get_tx_hash(), 0), UTXO{output: signed_tx.transaction.outputs[0].clone(), used_in_mempool: false});
        mempool.add_utxo((signed_tx.get_tx_hash(), 1), UTXO{output: signed_tx.transaction.outputs[1].clone(), used_in_mempool: false});
        mempool.add_utxo((signed_tx.get_tx_hash(), 2), UTXO{output: signed_tx.transaction.outputs[2].clone(), used_in_mempool: false});
        // compose a new tx which spend utxo1 and utxo2 and utxo3, and send 150 btc to pk b, and 10 btc to pk a
        let mut outputs = Vec::new();
        outputs.push(Output{ pk_hash: key_b.public_key().as_ref().to_vec().hash(), value: 140});
        outputs.push(Output{ pk_hash: key_a.public_key().as_ref().to_vec().hash(), value: 10});
        let mut inputs = Vec::new();
        inputs.push(Input{source_tx_hash: signed_tx.get_tx_hash(), index: 0});
        inputs.push(Input{source_tx_hash: signed_tx.get_tx_hash(), index: 1});
        inputs.push(Input{source_tx_hash: signed_tx.get_tx_hash(), index: 2});
        let tx = Transaction{inputs: inputs, outputs: outputs};
        let mut witnesses = Vec::new();
        let sig1 =sign(&tx, &key_a);
        let sig2 =sign(&tx, &key_a);
        let sig3 =sign(&tx, &key_b);
        let witness1 = Witness{pubkey: key_a.public_key().as_ref().to_vec() as PublicKey, sig: sig1.as_ref().to_vec()};
        let witness2 = Witness{pubkey: key_a.public_key().as_ref().to_vec() as PublicKey, sig: sig2.as_ref().to_vec()};
        let witness3 = Witness{pubkey: key_b.public_key().as_ref().to_vec() as PublicKey, sig: sig3.as_ref().to_vec()};
        witnesses.push(witness1);
        witnesses.push(witness2);
        witnesses.push(witness3);
        let tx_tobe_add = SignedTransaction{transaction: tx, fee: 10, witnesses:witnesses};
        // add this signed tx, the result must be ok
        let result = mempool.add_tx(&tx_tobe_add);
        assert_eq!(result.is_ok(), true);
        // build a block which contain a tx which spend utxo1 and utxo3, and send 90 btc to pk b, and 10 btc to pk a, 10 to fee
        let mut outputs = Vec::new();
        outputs.push(Output{ pk_hash: key_b.public_key().as_ref().to_vec().hash(), value: 90});
        outputs.push(Output{ pk_hash: key_a.public_key().as_ref().to_vec().hash(), value: 10});
        let mut inputs = Vec::new();
        inputs.push(Input{source_tx_hash: signed_tx.get_tx_hash(), index: 0});
        inputs.push(Input{source_tx_hash: signed_tx.get_tx_hash(), index: 2});
        let tx = Transaction{inputs: inputs, outputs: outputs};
        let mut witnesses = Vec::new();
        let sig1 =sign(&tx, &key_a);
        let sig3 =sign(&tx, &key_b);
        let witness1 = Witness{pubkey: key_a.public_key().as_ref().to_vec() as PublicKey, sig: sig1.as_ref().to_vec()};
        let witness3 = Witness{pubkey: key_b.public_key().as_ref().to_vec() as PublicKey, sig: sig3.as_ref().to_vec()};
        witnesses.push(witness1);
        witnesses.push(witness3);
        let tx = SignedTransaction{transaction: tx, fee: 10, witnesses:witnesses};
        // build a block comtain this tx 
        let mut txs = Vec::new();
        txs.push(tx);
        let blk = Block::new_block_from_txs(&H256::default(), &txs); 
        // add the block to the mempool by call receive finalized block
        let result = mempool.receive_finalized_block(&blk);
        assert_eq!(result.is_ok(), true);
        // check if the utxo are removed from the mempool
        let utxo1 = mempool.utxo.get(&(signed_tx.get_tx_hash(), 0));
        assert_eq!(utxo1.is_none(), true);
        let utxo2 = mempool.utxo.get(&(signed_tx.get_tx_hash(), 1));
        assert_eq!(utxo2.is_none(), false);
        let utxo3 = mempool.utxo.get(&(signed_tx.get_tx_hash(), 2));
        assert_eq!(utxo3.is_none(), true);
        // check if tx is removed 
        assert_eq!(mempool.txs.len(), 0);
        // check the outputs from the new block are added in utxo
        assert_eq!(mempool.utxo.len(), 3);
    }
}