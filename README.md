# RBTC 

An minimalistic bitcoin-like blockchain implemented in Rust: 

- Segwit UTXO model, which implement P2PWPKH-like payment. The input redeem code `pk, sig` are moved in witness. 
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
- K blocks confirmation

Todos: 
- [ ] CLI wallet which manage the keys and construct transactions
- [ ] user-chain Test 
- [ ] Doc


> This repo is fork from the repository for COS/ECE 470: Principles of Blockchains, Fall 2022 at Princeton University. [Main website of the course](https://blockchains.princeton.edu/principles-of-blockchains/).

