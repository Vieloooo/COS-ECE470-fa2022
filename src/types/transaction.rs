use serde::{Serialize,Deserialize};
use ring::signature; 
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::Rng;
use bincode;
use super::hash::{Hashable, H256};
use super::key_pair::PublicKey;
/// A simple bitcoin like utxo transaction model (no script) using p2pkh (pay to public key hash)
/// input: (pre_block_hash, pre_tx_hash, tx_index, publckey, signature)
/// output: (address, value)
/// validate input 
///     1. check if the input is in the utxo set
///     2. check for each input if the signature is valid
///         1. sig =? sig((pre_input_hash, outputs), pk)
///         2. check if the pk is the pk in the output 
///     3. check sum(input) > sum(output)
/// 
/// Input 
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Input {
    /// The hash of the transaction that contains the UTXO we want to spend
    pub source_tx_hash: H256,
    /// The index of the UTXO in the source transaction's output list
    pub index: usize,
}
impl Input {
    pub fn new(source_tx_hash: &H256, index: usize) -> Self {
        Input {
            source_tx_hash: source_tx_hash.clone(), 
            index: index, 
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Output{
    pub pk_hash: H256, 
    pub value: u64,
}
/// witness 
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Witness{
    pub pubkey : PublicKey, 
    pub sig: Vec<u8>
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    /// Inputs
    pub inputs: Vec<Input>,
    /// Outputs
    pub outputs: Vec<Output>, 

}
impl Hashable for Transaction{
    fn hash(&self) -> H256 {
        // First, we serialize the tx into bytes using bitnodes 
        // Then, we hash the bytes using ring::digest::digest
        let tx_bytes = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &tx_bytes).into()
    }
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    /// Transaction
    pub transaction: Transaction,
    /// Tx fee = sum input - sum outputs 
    pub fee: u32, 
    /// sigature list for each input 
    pub witnesses: Vec<Witness>,
}
impl SignedTransaction{
    /// For a given transaction, given the corresbonding outputs, verify the transaction, return the fee if valid, otherwise return -1, fee >=0 is valid.
    /// 
    pub fn verify(&self, receiver_outputs: &Vec<Output>) -> i64 {
        // check the witness length is the same as the input length
        if self.transaction.inputs.len() != self.witnesses.len(){
            return -1; 
        }
        // verify the public key in the witness is correct 
        let l = self.transaction.inputs.len();
        for i in 0..l{
            let wit = &self.witnesses[i];
            let input = &self.transaction.inputs[i];
            let output_hash = &receiver_outputs[input.index].pk_hash;
            if wit.pubkey.hash() != *output_hash {
                return -1; 
            }
        }
        // for each sig witness in the witness list, verify the signature 
        for wit in &self.witnesses{
            if !verify(&self.transaction, &wit.pubkey, &wit.sig) {
                return -1; 
            }
        }
        // verify the fee 
        let mut res:i64 = 0; 
        for income in receiver_outputs{
            res += income.value as i64;
        }
        for outcome in &self.transaction.outputs{
            res -= outcome.value as i64;

        }
        if res != self.fee as i64{
            return -1; 
        }
        res
    }
    pub fn get_tx_hash(&self) -> H256 {
        self.transaction.hash()
    }
    pub fn get_wtxid(&self) -> H256 {
        self.hash()
    }
}

impl Hashable for SignedTransaction{
    fn hash(&self) -> H256 {
        // First, we serialize the tx into bytes using bitnodes 
        // Then, we hash the bytes using ring::digest::digest
        let tx_bytes = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &tx_bytes).into()
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    //unimplemented!()
    // reference: https://docs.rs/ring/latest/ring/signature/index.html#signing-and-verifying-with-ed25519
    //convert tx to bytes slice 
    let tx_bytes = bincode::serialize(&t).unwrap();
    key.sign(&tx_bytes)


}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    // reference: https://docs.rs/ring/latest/ring/signature/index.html#signing-and-verifying-with-ed25519

    let pub_key =   signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
    //convert tx to bytes slice
    let tx_bytes = bincode::serialize(&t).unwrap();
    pub_key.verify(&tx_bytes, signature).is_ok()
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    let inputs = Vec::new();
    let outputs = Vec::new();
    Transaction{inputs: inputs, outputs: outputs}

}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;


    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        //test if the signature is valid for the same transaction but different key, this should not pass the verification 
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
        
        
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST