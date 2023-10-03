use serde::{Serialize,Deserialize};
use ring::signature; 
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::Rng;
use bincode;
use super::address::Address;
use super::hash::{Hashable, H256};
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    /// Sender's address
    pub sender: Address, 
    /// Receiver's address
    pub receiver: Address,
    /// tx's value 
    pub value: u64, 

}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    /// Transaction
    pub transaction: Transaction,
    /// Signature saved in vector
    pub signature: Vec<u8>,
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
    //unimplemented!()
    // reference: https://docs.rs/ring/latest/ring/signature/index.html#signing-and-verifying-with-ed25519
    let pub_key =   signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
    //convert tx to bytes slice
    let tx_bytes = bincode::serialize(&t).unwrap();
    pub_key.verify(&tx_bytes, signature).is_ok()
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    //unimplemented!()
    let mut rng = rand::thread_rng();
    let sender = Address::from(rng.gen::<[u8; 20]>());
    let receiver = Address::from(rng.gen::<[u8; 20]>());
    let value = rng.gen::<u64>();
    Transaction { sender, receiver, value }
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