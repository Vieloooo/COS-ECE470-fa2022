pub mod worker;

use log::info;
use std::sync::{Arc, Mutex};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;

use std::thread;
use crate::types::hash::{Hashable, H256};
use crate::types::block::Block;
use crate::Blockchain;
use crate::types::block::generate_random_block;

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update, // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
    Idle, 
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
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(blockchain:&Arc<Mutex<Blockchain>> ) -> (Context, Handle, Receiver<Block>) {
    // api_server => miner_thread 
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    // miner_thread => miner_worker_thread 
    let (finished_block_sender, finished_block_receiver) = unbounded();
    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
        blockchain:Arc::clone(blockchain),
        last_block_hash: blockchain.lock().unwrap().tip(),
    };
    //a sender abstraction for control signal from api server 
    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_block_receiver)
}

#[cfg(any(test,test_utilities))]
fn test_new() -> (Context, Handle, Receiver<Block>) {
   let blockchain = Arc::new(Mutex::new(Blockchain::new()));
   println!("The genesis hash is {:?}", blockchain.lock().unwrap().tip());
    new(&blockchain)
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
                OperatingState::Idle => {
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
                            self.last_block_hash = self.blockchain.lock().unwrap().tip();
                            println!("Updated: The lastest block hash is {:?}", self.last_block_hash);
                        }
                    };
                }
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
                            println!("Updated: The lastest block hash is {:?}", self.last_block_hash);
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

            // TODO for student: actual mining, create a block
            println!("Mining a new block"); 
            let mut blockchain = self.blockchain.lock().unwrap();
            //get the tip of the blockchain
            //let tip = blockchain.tip();
            let mut new_block = generate_random_block(&self.last_block_hash);
            new_block.header.nonce = 0;
            //get the difficulty of the tip
            new_block.header.difficulty = blockchain.blocks.get(&self.last_block_hash).unwrap().block.get_difficulty();
            // range nounce from 0 to u32::max_value()
            loop {
                if new_block.hash() < new_block.header.difficulty {
                    break;
                }
                new_block.header.nonce += 1;
            }
            println!("mined a new block, hash is {:?}", new_block.hash());
            self.operating_state = OperatingState::Idle;

            // TODO for student: if block mining finished, you can have something like: self.finished_block_chan.send(block.clone()).expect("Send finished block error");
            blockchain.insert(&new_block);
            self.finished_block_chan.send(new_block.clone()).expect("Send finished block error");

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
    use ntest::timeout;
    use crate::types::hash::Hashable;

    #[test]
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