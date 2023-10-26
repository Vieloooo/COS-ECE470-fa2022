pub mod worker;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::debug;
use log::info;
use std::sync::{Arc, Mutex};
use std::time;

use crate::types::block;
use crate::types::block::generate_random_block;
use crate::types::block::Block;
use crate::types::block_buffer;
use crate::types::hash::{Hashable, H256};
use crate::types::mempool::Mempool;
use crate::Blockchain;
use std::thread;
enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update,     // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    finished_block_chan: Sender<Block>,
    /// The whole blockchain of this user
    blockchain: Arc<Mutex<Blockchain>>,
    // lastest block hash
    last_block_hash: H256,
    //mempool
    mempool: Arc<Mutex<Mempool>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
) -> (Context, Handle, Receiver<Block>) {
    // api_server => miner_thread
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    // miner_thread => miner_worker_thread
    let (finished_block_sender, finished_block_receiver) = unbounded();
    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
        blockchain: Arc::clone(blockchain),
        last_block_hash: blockchain.lock().unwrap().tip(),
        mempool: Arc::clone(mempool),
    };
    //a sender abstraction for control signal from api server
    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_block_receiver)
}

#[cfg(any(test, test_utilities))]
fn test_new() -> (Context, Handle, Receiver<Block>) {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let mempool = Arc::new(Mutex::new(Mempool::new()));
    println!("The genesis hash is {:?}", blockchain.lock().unwrap().tip());
    new(&blockchain, &mempool)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    /// Start a miner thread which running the mining loop
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn miner_loop(&mut self) {
        // main mining loop
        loop {
            // check and react to control signals
            match self.operating_state {
                // if the miner in paused state, it will wait for the signal from api server, then go to the next loop
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Miner shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Miner starting in continuous mode with lambda {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Update => {
                            // in paused state, don't need to update
                        }
                    };
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                // working state
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        match signal {
                            ControlSignal::Exit => {
                                info!("Miner shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Miner starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                //unimplemented!()
                                self.last_block_hash = self.blockchain.lock().unwrap().tip();
                                info!(
                                    "Updated: The lastest block hash is {:?}",
                                    self.last_block_hash
                                );
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }

            if let OperatingState::ShutDown = self.operating_state {
                return;
            } 
            let mut new_block: Block;
            // get the new block body 
            let (new_body, mr, fee) = self.mempool.lock().unwrap().propose_block_body();

            //get header 
            let mut new_header = block::generate_random_header(& H256::default());
            let difficulty: H256;
            {
                //get the last hash and difficulty of the blockchain
               
                difficulty = self.blockchain.lock().unwrap().get_difficulty();
                self.last_block_hash = self.blockchain.lock().unwrap().tip();
            }
            new_header.difficulty = difficulty;
            new_header.merkle_root = mr;
            new_header.parent = self.last_block_hash;
            // build block 
            new_block = Block {
                header: new_header,
                body: new_body,
            };
            debug!("Start mining a block from {:?}", self.last_block_hash);
            info!("The new block will wrap {} tx", new_block.body.tx_count);

            // range nounce 
            loop {
                if new_block.hash() < new_block.header.difficulty {
                    break;
                }
                new_block.header.nonce += 1;
            }

            // push to the chain 
            info!("mined a new block, hash is {:?}", new_block.hash());
            let mut blockchain_unlocked = self.blockchain.lock().unwrap();
            let mut unlocked_mempool = self.mempool.lock().unwrap();
            block_buffer::blockchain_insert_with_mempool_atomic(new_block.clone(),&mut blockchain_unlocked, &mut unlocked_mempool);



            self.finished_block_chan
                .send(new_block.clone())
                .expect("Send finished block error");

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use crate::types::hash::Hashable;
    use ntest::timeout;

    //#[test]
    #[timeout(60000)]
    fn miner_three_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        // This test
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        miner_handle.update();
        for _ in 0..2 {
            let block_next = finished_block_chan.recv().unwrap();
            miner_handle.update();
            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST

// node1: http://127.0.0.1:7000/miner/start?lambda=1500000 1.5 sec per block
// node2: http://127.0.0.1:7001/miner/start?lambda=2000000 2 sec per block
// node3: http://127.0.0.1:7002/miner/start?lambda=2000000 2 sec per block
