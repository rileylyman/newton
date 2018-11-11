use super::Hashable;
use std::fmt;

enum MerkleBranch<T : Hashable> {
    Branch(Box<MerkleTree<T>>),
    Leaf(T, String),
    None
}

pub struct MerkleTree<T : Hashable> {
    left: MerkleBranch<T>,
    right: MerkleBranch<T>,
    mrkl_root: String,
    depth: usize
}

pub enum MrklVR { //Merkle Validation Result
    Valid,
    InvalidHash,
    InvalidTree
}

impl<T: Hashable> MerkleTree<T> {

    fn construct_leaf(data: &mut Vec<T>, hash: &mut String) -> MerkleBranch<T> {
            
            let first = data.remove(0);
            let first_hash = first.get_hash();
            
            hash.push_str(&first_hash);

            MerkleBranch::Leaf(first, first_hash)
    }

    fn construct_branch(data: &mut Vec<MerkleTree<T>>, hash: &mut String) -> MerkleBranch<T> {
        
        let first = data.remove(0);
        hash.push_str(&first.mrkl_root);

        MerkleBranch::Branch(Box::new(first))
    }

    fn construct_fringe_node(data: &mut Vec<T>) -> MerkleTree<T> {    
       
        let mut hash = String::new();

        let left_leaf = MerkleTree::construct_leaf(data, &mut hash);

        let mut right_leaf = MerkleBranch::None;
        if data.len() > 0 {
            
            right_leaf = MerkleTree::construct_leaf(data, &mut hash);
        }
        
        hash = hash.get_hash();

        MerkleTree{
            left: left_leaf,
            right: right_leaf,
            mrkl_root: hash,
            depth: 0
        }
    }

    fn construct_internal_node(data: &mut Vec<MerkleTree<T>>, depth: usize) -> MerkleTree<T> {
        let mut hash = String::new();

        let left_branch = MerkleTree::construct_branch(data, &mut hash);

        let mut right_branch = MerkleBranch::None;
        if data.len() > 0 {
            right_branch = MerkleTree::construct_branch(data, &mut hash);
            hash = hash.get_hash();   
        }
        MerkleTree{
            left: left_branch,
            right: right_branch,
            mrkl_root: hash,
            depth
        }
    }

    pub fn construct(data: &mut Vec<T>) -> Self {
        assert!(data.len() > 1);

        let mut mrkl_trees: Vec<MerkleTree<T>> = Vec::new();

        while data.len() > 0 {

            let fringe_node = MerkleTree::construct_fringe_node(data);
            mrkl_trees.push(fringe_node);

        }

        let mut depth = 1;

        while mrkl_trees.len() > 1 {

            let mut new_mrkl_trees: Vec<MerkleTree<T>> = Vec::new();

            while mrkl_trees.len() > 0 {

                let internal_node = MerkleTree::construct_internal_node(&mut mrkl_trees, depth);
                new_mrkl_trees.push(internal_node);
            }

            mrkl_trees = new_mrkl_trees;
            depth += 1;        
        }
        mrkl_trees.remove(0)
    }

    pub fn validate(&self) -> MrklVR { //TODO: add depth checking
       
        match (&self.left, &self.right) {
           
           (MerkleBranch::Branch(ref left_br), MerkleBranch::Branch(ref right_br)) => {
               
                match (left_br.validate(), right_br.validate()) {
                    
                    (MrklVR::Valid, MrklVR::Valid) => {

                        //Check that current node hash is same as computed hash
                        let mut hash = String::new();
                        hash.push_str(&left_br.mrkl_root);
                        hash.push_str(&right_br.mrkl_root);

                        hash = hash.get_hash();
                        
                        if hash == self.mrkl_root { MrklVR::Valid }
                        else {
                            debug_assert!(false, "On internal node: mrkl_root differs from expected."); 
                            MrklVR::InvalidHash
                        }
                    }
                    (MrklVR::InvalidHash, MrklVR::InvalidHash) => MrklVR::InvalidHash,
                    (MrklVR::InvalidHash, _) => MrklVR::InvalidHash,
                    (_, MrklVR::InvalidHash) => MrklVR::InvalidHash,
                    (_,_) => MrklVR::InvalidTree,
                }
            }
            (MerkleBranch::Leaf(ref left_it, ref left_hash), MerkleBranch::Leaf(ref right_it, ref right_hash)) => {
                
                let mut hash  = String::new();
                hash.push_str( left_hash);
                hash.push_str(right_hash);
                
                hash = hash.get_hash();
                
                if  left_it.get_hash() == *left_hash && 
                    right_it.get_hash() == *right_hash &&
                    self.mrkl_root == hash {
                    
                    MrklVR::Valid
                } else if self.mrkl_root != hash {
                   
                    debug_assert!(false, "On leaf node: mrkl_root does not match concatenated hash.");
                    MrklVR::InvalidHash
                } else {

                    debug_assert!(false, "On leaf node: leaf hash does not equal expected leaf hash");
                    MrklVR::InvalidHash
                }
            }
            (MerkleBranch::Branch(ref branch), MerkleBranch::None) => {
                if branch.mrkl_root == self.mrkl_root {
                    MrklVR::Valid
                } else {
                    debug_assert!(false, "On internal node: mrkl_root does not match only child\'s root.");
                    MrklVR::InvalidHash
                }
            }
            (MerkleBranch::Leaf(ref left_it, ref left_hash), MerkleBranch::None) => {
                
                let mut hash = left_it.get_hash();
                
                if &hash == left_hash && hash == self.mrkl_root {
                    
                    MrklVR::Valid
                } else {
                    
                    debug_assert!(false, "On lonely leaf node: hash does not match.");
                    MrklVR::InvalidHash
                }
            
            }
            (_,_) => {

                let mut err_msg = String::new();
                err_msg.push_str("Mismatched branch and leaf children for node: ");
                
                match &self.left {
                    MerkleBranch::Branch(_) => err_msg.push_str("Left child was a branch and "),
                    MerkleBranch::Leaf(_, _) => err_msg.push_str("Left child was a leaf and "),
                    MerkleBranch::None => err_msg.push_str("Left child was none and ")
                }
                match &self.right {
                    MerkleBranch::Branch(_) => err_msg.push_str("right child was a branch"),
                    MerkleBranch::Leaf(_, _) => err_msg.push_str("right child was a leaf"),
                    MerkleBranch::None => err_msg.push_str("right child was none")
                }

                debug_assert!(false, err_msg);

                MrklVR::InvalidTree
            }
        }
    }
}

impl<T: Hashable> fmt::Display for MerkleTree<T> {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_helper(f, 0, true)
    }
}

impl<T: Hashable> MerkleTree<T> {
    fn fmt_helper(&self, f: &mut fmt::Formatter, num_iter: u32, new_line: bool) -> fmt::Result {
        
        if new_line { for _ in 0..self.depth { write!(f, "    "); } }
        else { write!(f, "    "); }

        write!(f, "{}", self.depth.to_string());
        
        for _ in 0..(num_iter) {
             write!(f, "    ");
             write!(f, "{}", self.depth.to_string());
        }

        if new_line { write!(f, "\n"); }

        match (&self.left, &self.right) {
            (MerkleBranch::Branch(l_tree), MerkleBranch::Branch(r_tree)) => {
                l_tree.fmt_helper(f, num_iter + 1, false).unwrap();
                r_tree.fmt_helper(f, num_iter + 1, true).unwrap();
            }
            (_, _) => {}
        }

        Ok( () )
    }

}
