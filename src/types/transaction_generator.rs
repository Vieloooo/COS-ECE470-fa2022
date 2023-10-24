use std::{thread, time};
use crate::network::server::Handle as NetworkServerHandle;
use std::sync::{Arc, Mutex};
use crate::types::mempool::Mempool;
use crate::types::transaction::*; 
use crate::types::key_pair; 
pub struct TransactionGenerator{}

impl TransactionGenerator{
    pub fn start(theta: u32, network: NetworkServerHandle, mempool : Arc<Mutex<Mempool>>){
        thread::spawn(move || loop{
            if theta != 0{
                let interval = time::Duration::from_micros(theta as u64);
                thread::sleep(interval);
            }
        });
    }
}