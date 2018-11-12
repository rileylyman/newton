/*!
 * A Merkle Tree implementation. 
 * 
 * # Errors
 * Constructing a Merkle Tree using `MerkleTree::construct(&mut Vec<T>)` will return
 * an error result if the passed vector has fewer than two items.
 * 
 * # Panics
 * - In non-release builds, constructing a Merkle Tree will panic if we call the constructor
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
use self::MrklVR::*;

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
 * `height`: The height of the current node in the overall `MerkleTree`. Leaves have height 0.
 */
pub struct MerkleTree<T : Hashable> {
    left: MerkleBranch<T>,
    right: MerkleBranch<T>,
    mrkl_root: String,
    height: usize
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

        let mut height = 1;

        while mrkl_trees.len() > 1 {

            let mut new_mrkl_trees: Vec<MerkleTree<T>> = Vec::new();

            while mrkl_trees.len() > 0 {

                let internal_node = MerkleTree::construct_internal_node(&mut mrkl_trees, height);
                new_mrkl_trees.push(internal_node);
            }

            mrkl_trees = new_mrkl_trees;
            height += 1;        
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
                    
                    (Valid, Valid) => self.validate_internal_node(&left_br, Some(&right_br)),

                    (result @ InvalidHash, _) | (_, result @ InvalidHash) => result,

                    (_,_) => InvalidTree,
                }
            }
            (MerkleBranch::Branch(ref branch), MerkleBranch::None) => {

                match branch.validate() {
                    Valid => self.validate_internal_node(branch, None),
                    result @ InvalidHash | result @ InvalidTree => result
                }
                
            }
            (MerkleBranch::Leaf(ref left_it, ref left_hash), MerkleBranch::Leaf(ref right_it, ref right_hash)) 
                    => self.validate_fringe_node(left_it, left_hash, Some(right_it), Some(right_hash)),
            
            (MerkleBranch::Leaf(ref left_it, ref left_hash), MerkleBranch::None) 
                    => self.validate_fringe_node(left_it, left_hash, None, None),
                    
            (_,_) => {
                debug_assert!(false, "Mismatched children for node.");
                InvalidTree
            }
        }
    }


    /*
    --------------------------------------------------------------------------------------------------------
    |                                   Private MerkleTree methods below                                   |
    --------------------------------------------------------------------------------------------------------
    */


    /**
     * Helper function for `MerkleTree::Validate` which validates an internal node in the Merkle tree.
     * It first computes the concatenated hash for its two children, and compares that with its
     * `mrkl_root`. It then checks that the height of its children are one less than its height.
     */
    fn validate_internal_node(&self, left_node: &MerkleTree<T>, right_node: Option<&MerkleTree<T>>) -> MrklVR {

        let mut hash = String::new();
        hash.push_str(&left_node.mrkl_root);

        let mut right_has_correct_height = true;
        match right_node {

            Some(r) => {
                hash.push_str(&r.mrkl_root);
                hash = hash.get_hash();

                right_has_correct_height = self.height == r.height + 1;
            }

            None => {}
        }
    
        if hash == self.mrkl_root && 
           self.height == left_node.height + 1 &&
           right_has_correct_height
        { 
               Valid 
        }
        else if self.height != left_node.height + 1 ||
                right_has_correct_height
        {
            debug_assert!(false, "Mismatched heights for internal node.");
            InvalidTree
        } 
        else {
            debug_assert!(false, "On internal node: mrkl_root differs from expected."); 
            InvalidHash
        }
    }

    /**
     * Helper function for `MerkleTree::Validate` which validates a fringe node in the Merkle tree.
     * It first computes the concatenated hash for its children, and compares that with its
     * `mrkl_root`. It then checks that its height is 0.
     */
    fn validate_fringe_node(&self, left_it: &T, left_hash: &str, right_it: Option<&T>, right_hash: Option<&str>)
            -> MrklVR {
        
        let mut hash  = String::new();
        hash.push_str( left_hash);

        let mut right_hash_is_valid = true;
        match (right_it, right_hash) {

            (Some(r), Some(r_hash)) => {
                hash.push_str(&r_hash);
                hash = hash.get_hash();

                right_hash_is_valid = r.get_hash() == r_hash;
            }

            (_,_) => {}
        }
        
        
        if  left_it.get_hash() == *left_hash && 
            right_hash_is_valid &&
            self.mrkl_root == hash &&
            self.height == 0 {
            
            Valid
        } else if self.mrkl_root != hash {
           
            debug_assert!(false, "On leaf node: mrkl_root does not match concatenated hash.");
            InvalidHash
        }
        else if self.height != 0 {
            debug_assert!(false, "height is not zero on fringe node.");
            InvalidTree
        } else {
            debug_assert!(false, "On leaf node: leaf hash does not equal expected leaf hash");
            InvalidHash
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
            hash = hash.get_hash();
        }
        

        MerkleTree{
            left: left_leaf,
            right: right_leaf,
            mrkl_root: hash,
            height: 0
        }
    }

    /**
     * Helper function for `MerkleTree::construct`. Creates a `MerkleTree` from the first
     * two elements of `data`, where the children of this `MerkleTree` are other `MerkleTree`s. 
     */
    fn construct_internal_node(data: &mut Vec<MerkleTree<T>>, height: usize) -> MerkleTree<T> {
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
            height
        }
    }
}
