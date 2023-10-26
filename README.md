# RBTC 

An minimalistic bitcoin-like blockchain implemented in Rust: 

- Segwit UTXO model, which implement P2PWPKH-like payment. The input redeem code `pk, sig` are moved to the witness part. 
```Rust 
    pub struct SignedTransaction {
        /// Transaction
        pub transaction: Transaction,
        /// Tx fee 
        pub fee: u32, 
        /// sigature list for each input 
        pub witnesses: Vec<Witness>,
    }
    pub struct Transaction {
        /// Inputs
        pub inputs: Vec<Input>,
        /// Outputs
        pub outputs: Vec<Output>, 
    }
    pub struct Witness{
        pub pubkey : PublicKey, 
        pub sig: Vec<u8>
    }
```
- K-block confirmation/finalization. 
- UTXO and mempool are consistant with the finalized blocks. 

## Usage 

Build project: in `/wallet` and `/node` directory, run `bash build.sh`

Setup keypairs: 
- store your Ed25519 keys in `Document` [bytes](https://docs.rs/ring/latest/ring/signature/struct.Ed25519KeyPair.html). The test keys are stored in `/keys`.
- store the test public key hashes in `/pks.txt`,which provide quick pkh query. No need for typing the pkh in terminal again.  
Using wallet: run `./wallet --help`
```
RBTC Wallet 0.1.0
PlasticBug
Check you account and tranfer your RBTC!

USAGE:
    wallet [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --address <ADDRESS>    Sets the server address [default:
                               http://127.0.0.1:7000]
    -k, --key <FILE>           Sets the key file [default: ../keys/alice.key]
    -n, --neighbors <FILE>     Sets the neighbors file [default: ../pks.txt]

SUBCOMMANDS:
    help                Prints this message or the help of the given
                        subcommand(s)
    show_utxo_detail    Shows UTXO details
    transfer            Transfers x RBTC to pkh
    transfer_by_id      Transfers RBTC to an neighbor with index i
```

Launching node: run `./bitcoin --help`

```
$ ./bitcoin --help
RBTC 0.1
minalistic UTXO Blockchin in Rust

USAGE:
    bitcoin [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Increases the verbosity of logging

OPTIONS:
        --api <ADDR>           Sets the IP address and the port of the API server
                               [default: 127.0.0.1:7000]
    -c, --connect <PEER>...    Sets the peers to connect to at start
        --p2p-workers <INT>    Sets the number of worker threads for P2P server
                               [default: 4]
        --p2p <ADDR>           Sets the IP address and the port of the P2P server
                               [default: 127.0.0.1:6000]
```
## Tests
- Run uni-tests for node, run `cargo test`
- Run multi-node test
  - launch 3 nodes 
  - in `/tests`` dir, run all 3 `a2b.sh`, `b2c.sh`, `c2d.sh` files in 3 terminal. 
  - check the consistency of the finalized block of the three nodes. 



Todos: 

- [ ] add coinbase 


> This repo is fork from the [project repository](https://github.com/Blockchains-Princeton/COS-ECE470-fa2022) for the course COS/ECE 470: Principles of Blockchains, Fall 2022 at Princeton University. 

