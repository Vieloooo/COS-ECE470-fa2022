
use ring::signature::{Ed25519KeyPair, KeyPair};
use super::hash::Hashable;
use super::key_pair::{self, PublicKey};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path; 
pub struct IcoGenerator{}
pub const CONFIG_PATH :&str = "../keys"; 
pub struct Ico{
    pub alice: Ed25519KeyPair,
    pub bob: Ed25519KeyPair,
    pub caro: Ed25519KeyPair,
}

impl IcoGenerator{
    /// Gen 3 key pair
    /// Alice 1000000 btc 
    /// Bob 1000000 btc
    /// Caro 1000000 btc
    /// store them in config file
    pub fn new_key(config_path: &str) {
        let root_path = Path::new(config_path);
        let alice = key_pair::random_serialized();
        let bob = key_pair::random_serialized();
        let caro = key_pair::random_serialized();
        // for each key_pair, write them to alice.key, bob.key, caro.key
        let mut alice_file = fs::File::create(root_path.join("alice.key")).unwrap();
        alice_file.write_all(alice.as_ref()).unwrap();
        let mut bob_file = fs::File::create(root_path.join("bob.key")).unwrap();
        bob_file.write_all(bob.as_ref()).unwrap();
        let mut caro_file = fs::File::create(root_path.join("caro.key")).unwrap();
        caro_file.write_all(caro.as_ref()).unwrap();
        
       
        
    }
    pub fn load_key(config_path: &str) -> Vec<Ed25519KeyPair>{
        let root_path = Path::new(config_path);
        let mut alice_file = fs::File::open(root_path.join("alice.key")).unwrap();
        let mut alice = Vec::new();
        alice_file.read_to_end(&mut alice).unwrap();
        let mut bob_file = fs::File::open(root_path.join("bob.key")).unwrap();
        let mut bob = Vec::new();
        bob_file.read_to_end(&mut bob).unwrap();
        let mut caro_file = fs::File::open(root_path.join("caro.key")).unwrap();
        let mut caro = Vec::new();
        caro_file.read_to_end(&mut caro).unwrap();
        let alice = Ed25519KeyPair::from_pkcs8(&alice).unwrap();
        let bob = Ed25519KeyPair::from_pkcs8(&bob).unwrap();
        let caro = Ed25519KeyPair::from_pkcs8(&caro).unwrap();
        return Vec::from([alice, bob, caro]);

    }
    pub fn print_pkh(config_path: &str) {
        let keys = IcoGenerator::load_key(config_path);
        for key in keys {
            let pk: PublicKey = key.public_key().as_ref().to_vec();
            let pk_hash = pk.hash();
            println!("{}", pk_hash);
        }
    }
   
    
}

#[cfg(test)]
mod test{
    use super::*;
    use ring::signature::KeyPair;
    //#[test]
    fn test_ico_new_load_key(){
        IcoGenerator::new_key(CONFIG_PATH);
        let ico = IcoGenerator::load_key(CONFIG_PATH);
        let ico2 = IcoGenerator::load_key(CONFIG_PATH);
        assert_eq!(ico[0].public_key().as_ref(), ico2[0].public_key().as_ref());
        assert_eq!(ico[1].public_key().as_ref(), ico2[1].public_key().as_ref());
        assert_eq!(ico[2].public_key().as_ref(), ico2[2].public_key().as_ref());
    }

    #[test]
    fn test_ico_print_keys_pkh(){
        IcoGenerator::print_pkh(CONFIG_PATH);
    }
}
