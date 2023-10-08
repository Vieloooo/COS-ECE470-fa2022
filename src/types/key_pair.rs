use ring::rand;
use ring::signature::Ed25519KeyPair;
use super::hash::{H256, Hashable};

pub type PublicKey = Vec<u8>;

impl Hashable for PublicKey {
    fn hash(&self) -> H256 {
        ring::digest::digest(&ring::digest::SHA256, &self).into()
    }
}

/// Generate a random key pair.
pub fn random() -> Ed25519KeyPair {
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref().into()).unwrap()
}
