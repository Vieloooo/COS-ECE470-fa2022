use super::hash::{Hashable, H256};

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    /// Root of the Merkle Tree 
    root: H256,
    /// The number of leaves in the Merkle Tree
    leaf_size: usize, 
    /// Merkle tree
    merkle_tree: Vec<Vec<H256>>,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        //unimplemented!()
        // hash all elements and put them into leafs 
        let mut leafs = data.iter().map(|x| x.hash()).collect::<Vec<H256>>();
        // *get the total number of valid leafs, if the number of leafs is not a power of 2, padding the merkle tree with the last leaf 
        let leaf_size = leafs.len();
        // pad the leafs to be a even number 
        if leaf_size % 2 == 1 {
            leafs.push(leafs[leaf_size - 1]);
        }
        if leaf_size ==0{
            panic!("no leaf in the merkle tree");
        }
        let mut tree_hashes = Vec::new();
        let current_level : usize = 0; 
        tree_hashes.push(leafs);
        loop {
            let mut current_hashes = Vec::new();
            let current_level = current_level + 1;
            let mut i = 0;
            while i < tree_hashes[current_level - 1].len() {
                let mut hasher = ring::digest::Context::new(&ring::digest::SHA256);
                hasher.update(tree_hashes[current_level - 1][i].as_ref());
                hasher.update(tree_hashes[current_level - 1][i + 1].as_ref());
                let hash = hasher.finish();
                current_hashes.push(hash.into());
                i = i + 2;
            }
            let current_hashes_len = current_hashes.len();
            if current_hashes_len != 1 && current_hashes_len % 2 == 1 {
                current_hashes.push(current_hashes[current_hashes_len - 1]);
            }
            tree_hashes.push(current_hashes);
            if current_hashes_len == 1 {
                break;
            }
        }
        let root = tree_hashes[tree_hashes.len() - 1][0].clone();
        // return the merkle tree
        MerkleTree {
            root,
            leaf_size,
            merkle_tree: tree_hashes,
        }

    }

    pub fn root(&self) -> H256 {
        //unimplemented!()
        self.root
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        //unimplemented!()
        let mut proof = Vec::new();
        let mut current_level: usize = 0; 
        let mut current_index = index;
        let tree_depth = self.merkle_tree.len();
        loop {
            let sbling_index = get_sbling(current_index);
            let sbling_hash = self.merkle_tree[current_level][sbling_index].clone();
            proof.push(sbling_hash);
            current_index = current_index / 2;
            current_level = current_level + 1;
            if current_level == tree_depth - 1 {
                break;
            }
        }
        proof
    }
}
fn get_sbling(index: usize) -> usize {
    let mut sbling_index = 0;
    if index % 2 == 0 {
        sbling_index = index + 1;
    } else {
        sbling_index = index - 1;
    }
    sbling_index
}
/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    //unimplemented!()
    let mut current_hash = datum.clone();
    if index > leaf_size - 1 {
        return false;
    }
    let mut hasher = ring::digest::Context::new(&ring::digest::SHA256);
    // watch out, the hash order matters
    let mut current_index = index;
    for i in 0..proof.len() {
        if current_index % 2 == 0 {
            hasher.update(current_hash.as_ref());
            hasher.update(proof[i].as_ref());
        } else {
            hasher.update(proof[i].as_ref());
            hasher.update(current_hash.as_ref());
        }
        let hash = hasher.finish();
        current_hash = hash.into();
        hasher = ring::digest::Context::new(&ring::digest::SHA256);
        current_index = current_index / 2;
    }
    current_hash == *root
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use crate::types::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST