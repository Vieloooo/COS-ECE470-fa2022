use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::types::block::Block;
use crate::types::hash::{H256, Hashable};
use crate::types::block_buffer::BlockBuffer;
use crate::Blockchain; 
use std::os::linux::raw;
use std::sync::{Arc, Mutex};
use log::{debug, warn, error};
use crate::types::mempool::Mempool;
use std::thread;

#[cfg(any(test,test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test,test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    block_buffer: Arc<Mutex<BlockBuffer>>, 
    mempool: Arc<Mutex<Mempool>>,
}


impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        _blockchain: &Arc<Mutex<Blockchain>>,
        _block_buffer: &Arc<Mutex<BlockBuffer>>,
        _mempool: &Arc<Mutex<Mempool>>,
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(_blockchain),
            block_buffer: Arc::clone(_block_buffer),  
            mempool: Arc::clone(_mempool),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                // receive other nodes' new block hashes. 
                // if I don't have the block, send a request to get the block
                Message::NewBlockHashes(hashes) => {
                    if hashes.is_empty(){
                        continue;
                    }
                    debug!("NewBlockHashes: {:?}", hashes);
                    let mut blocks_I_dont_have = Vec::new();
                    for hash in hashes {
                        if !self.blockchain.lock().unwrap().blocks.contains_key(&hash) {
                            blocks_I_dont_have.push(hash.clone());
                        }
                    } 
                    peer.write(Message::GetBlocks(blocks_I_dont_have));
                }
                Message::GetBlocks(hashes) => {
                    if hashes.is_empty(){
                        continue;
                    }
                    debug!("GetBlocks: {:?}", hashes);
                    let mut blocks = Vec::new();
                    for hash in hashes {
                        if let Some(block) = self.blockchain.lock().unwrap().blocks.get(&hash) {
                            blocks.push(block.block.clone());
                        }
                    }
                    peer.write(Message::Blocks(blocks));
                }
                Message::Blocks(input_blocks) => {
                    if input_blocks.is_empty(){
                        continue;
                    }
                    debug!("Blocks: {:?}", input_blocks);
                    // filter the input blocks, remove the under_mined blocks
                    // get current difficulty
                    let dif = self.blockchain.lock().unwrap().get_difficulty();
                    let mut input_blocks = input_blocks;
                    input_blocks.retain(|block| block.header.difficulty >= dif);
                    
                    //remove duplicated blocks which we already have
                    let mut blocks_I_dont_have = Vec::new();
                    let mut orphan_blocks = Vec::new(); 
                    let mut new_block_hashes = Vec::new(); 
                    for block in input_blocks {
                        if !self.blockchain.lock().unwrap().blocks.contains_key(&block.hash()) {
                            new_block_hashes.push(block.hash());
                            blocks_I_dont_have.push(block);
                        }
                    }
                    //broadcast the newcomming block hashes  
                    if !new_block_hashes.is_empty() {
                        self.server.broadcast(Message::NewBlockHashes(new_block_hashes));
                    }
                    //  for each new block, send it to the block buffer, this buffer will handle the process of pushing the block to the blockchain
                    for block in blocks_I_dont_have {
                        // In this working thread, only have 1 working loop and 1 buffer, so buffer can borrow the blockchain, no need for clone  
                        let parent_hash = block.header.parent; 
                        let have_parents = self.block_buffer.lock().unwrap().send_block(block, &self.blockchain, &self.mempool);

                        // if the newcoming block is an orphan block, send a request to get the parent block
                        if !have_parents {
                            orphan_blocks.push(parent_hash);
                        }

                    }
                    // request the parent block of the orphan blocks 
                    if !orphan_blocks.is_empty() {
                        peer.write(Message::GetBlocks(orphan_blocks));
                    }
                    // boardcast new incoming blocks that I not have 
                }
                _ => unimplemented!(),
            }
        }
    }
}

#[cfg(any(test,test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>
}
#[cfg(any(test,test_utilities))]
impl TestMsgSender {
    fn new() -> (TestMsgSender, smol::channel::Receiver<(Vec<u8>, peer::Handle)>) {
        let (s,r) = smol::channel::unbounded();
        (TestMsgSender {s}, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}
#[cfg(any(test,test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let block_buffer = Arc::new(Mutex::new(BlockBuffer::new()));
    let mempool = Arc::new(Mutex::new(Mempool::new()));
    let worker = Worker::new(1, msg_chan, &server, &blockchain, &block_buffer, &mempool);
    let init_hash = blockchain.lock().unwrap().tip();
    worker.start(); 
    (test_msg_sender, server_receiver, vec![init_hash])
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        log::debug!("gen a rand block {:?}", random_block.hash());
        let mut peer_receiver = test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST