/*!
 * A Merkle Tree implementation. 
 * 
 * # Errors
 * Constructing a Merkle Tree using `MerkleTree::construct(&mut Vec<T>)` will return
 * an error result if the passed vector has fewer than two items.
 * 
 * # Panics
 * - In non-release builds, constructing a Merkle Tree will panic if we call the constructer
 * with a vector of fewer than two elements.
 * - In non-release builds, validating a Merkle Tree will panic on Invalid results.
 * 
 * # Examples
 * 
 * ```
 * let data = vec!("some", "sample", "data");
 * let mrkl_tree = MerkleTree::construct(data);
 * assert_eq!(mrkl_tree.validate(), MrklVR::Valid);
 * 
 * ```
 *  
 */

use super::Hashable;

/**
 * An enumerations of children types for `MerkleTree`.
 * ---
 * When a child contains another `MerkleTree`, it is specified as `MerkleBranch::Branch`.
 * 
 * When a child is a leaf, it is specified as `MerkleBranch::Leaf`. Leaves contain 
 * an object of type `T` and a `String` which is the sha2 hash of that object.  
 * 
 * A child can also be `MerkleBranch::None`, if it contains no information.
 */
enum MerkleBranch<T : Hashable> {
    Branch(Box<MerkleTree<T>>),
    Leaf(T, String),
    None
}

/**
 * A struct representing a Merkle Tree, which may or may not be an internal node.
 * 
 * # Fields
 * `left`: The left child of the `MerkleTree`, held within a `MerkleBranch` enumeration.
 * 
 * `right`: The right child of the `MerkleTree`.
 * 
 * `mrkl_root`: The sha2 hash of the concatenation of the hashes of this `MerkleTree`'s children.
 * 
 * `depth`: The depth of the current node in the overall `MerkleTree`. Counterintuitvely, 
 * leaves have depth `0`.
 */
pub struct MerkleTree<T : Hashable> {
    left: MerkleBranch<T>,
    right: MerkleBranch<T>,
    mrkl_root: String,
    depth: usize
}

/**
 * The Merkle Validation Result enumeration enumerates the possible results of calling
 * `MerkleTree::validate` on a Merkle tree.
 * 
 * The result is `Valid` if there are no inconsistencies when validating the tree.
 * 
 * `InvalidHash` represents a situation when the hash of the children of a `MerkleTree`
 * do not equal the tree's `mrkl_root`. 
 * 
 * `InvalidTree` represents a situation where the given `MerkleTree` is malformed. For example,
 * its left child is a leaf and its right child is a branch.
 */
pub enum MrklVR {
    Valid,
    InvalidHash,
    InvalidTree
}

impl<T: Hashable> MerkleTree<T> {


    /**
     * Constructs a `MerkleTree` instance.
     * 
     * # Arguments
     * - `mut data: Vec<T>`: A vector of data which will be used to build the `MerkleTree` instance.
     * 
     * # Panics
     * In non-release builds, will panic if `data.len()` is less than 2. 
     * 
     * # Errors
     * Will return an error result if the length of `data` is less than 2. 
     */
    pub fn construct(mut data: Vec<T>) -> Result<Self, String> {
        if data.len() < 1 {
            debug_assert!(false, "Wrong number of arguments to merkle tree constructor.");

            return Err(String::from(
                "Not enough data to construct Merkle Tree. Must receive at least two items."
            ));
        }

        let mut mrkl_trees: Vec<MerkleTree<T>> = Vec::new();

        while data.len() > 0 {

            let fringe_node = MerkleTree::construct_fringe_node(&mut data);
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
        Ok(mrkl_trees.remove(0))
    }

    /**
     * Validates a given instance of `MerkleTree`.
     * 
     * # Return Value
     * Returns a `MrklVR` enumeration. See the documentation for `MrklVR` for the meanings
     * of each result.
     * 
     * # Panics
     * In non-release builds, will panic if the computer validation result is not `MrklVR::Valid`. 
     */
    pub fn validate(&self) -> MrklVR {
       
        match (&self.left, &self.right) {
           
           (MerkleBranch::Branch(ref left_br), MerkleBranch::Branch(ref right_br)) => {
               
                match (left_br.validate(), right_br.validate()) {
                    
                    (MrklVR::Valid, MrklVR::Valid) => {

                        //Check that current node hash is same as computed hash
                        let mut hash = String::new();
                        hash.push_str(&left_br.mrkl_root);
                        hash.push_str(&right_br.mrkl_root);

                        hash = hash.get_hash();
                        
                        if hash == self.mrkl_root && 
                           self.depth == left_br.depth + 1 &&
                           self.depth == right_br.depth + 1 
                        { 
                               MrklVR::Valid 
                        }
                        else if self.depth != left_br.depth + 1 ||
                                self.depth != right_br.depth + 1
                        {
                            debug_assert!(false, "Mismatched depths for internal node.");
                            MrklVR::InvalidTree
                        } 
                        else {
                            debug_assert!(false, "On internal node: mrkl_root differs from expected."); 
                            MrklVR::InvalidHash
                        }
                    }
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
                    self.mrkl_root == hash &&
                    self.depth == 0 {
                    
                    MrklVR::Valid
                } else if self.mrkl_root != hash {
                   
                    debug_assert!(false, "On leaf node: mrkl_root does not match concatenated hash.");
                    MrklVR::InvalidHash
                }
                else if self.depth != 0 {
                    debug_assert!(false, "Depth is not zero on fringe node.");
                    MrklVR::InvalidTree
                } else {

                    debug_assert!(false, "On leaf node: leaf hash does not equal expected leaf hash");
                    MrklVR::InvalidHash
                }
            }
            (MerkleBranch::Branch(ref branch), MerkleBranch::None) => {
                if branch.mrkl_root == self.mrkl_root && self.depth == branch.depth + 1 {
                    MrklVR::Valid
                }
                else if branch.depth + 1 != self.depth {
                    debug_assert!(false, "Depth mismatch.");
                    MrklVR::InvalidTree
                } else {
                    debug_assert!(false, "On internal node: mrkl_root does not match only child\'s root.");
                    MrklVR::InvalidHash
                }
            }
            (MerkleBranch::Leaf(ref left_it, ref left_hash), MerkleBranch::None) => {
                
                let mut hash = left_it.get_hash();
                
                if &hash == left_hash && hash == self.mrkl_root && self.depth == 0{
                    
                    MrklVR::Valid
                } else if self.depth != 0 {
                    debug_assert!(false, "Nonzero depth for fringe node.");
                    MrklVR::InvalidTree
                } else {
                    
                    debug_assert!(false, "On lonely leaf node: hash does not match.");
                    MrklVR::InvalidHash
                }
            
            }
            (_,_) => {
                debug_assert!(false, "Mismatched children for node.");

                MrklVR::InvalidTree
            }
        }
    }


    /**
     * Helper function for `MerkleTree::construct`. Pops off the first element of 
     * `data` and creates a `MerkleBranch::Leaf`. It also pushes the hash of this first element
     * into `hash`.
     */
    fn construct_leaf(data: &mut Vec<T>, hash: &mut String) -> MerkleBranch<T> {
            
            let first = data.remove(0);
            let first_hash = first.get_hash();
            
            hash.push_str(&first_hash);

            MerkleBranch::Leaf(first, first_hash)
    }

    /**
     * Helper function for `MerkleTree::construct`. Pops off the first element of `data`
     * and creates a `MerkleBranch::Branch`. Also pushes the hash of this first element
     * onto `hash`.
     */
    fn construct_branch(data: &mut Vec<MerkleTree<T>>, hash: &mut String) -> MerkleBranch<T> {
        
        let first = data.remove(0);
        hash.push_str(&first.mrkl_root);

        MerkleBranch::Branch(Box::new(first))
    }

    /**
     * Helper function for `MerkleTree::construct`. Creates a `MerkleTree` from the 
     * first two elements of `data`, where the children of this `MerkleTree` are
     * leaves.
     */
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

    /**
     * Helper function for `MerkleTree::construct`. Creates a `MerkleTree` from the first
     * two elements of `data`, where the children of this `MerkleTree` are other `MerkleTree`s. 
     */
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
}
