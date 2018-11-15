/*!
 * A Merkle Tree implementation. Currently supports: 
 * - Construction from a vector of objects
 * - `O(log n)` containment checks
 * - Pruning 
 * - Validation and pruned validation
 * 
 * # Errors
 * Constructing a Merkle Tree using `MerkleTree::construct(&mut Vec<T>)` will return
 * an error result if the passed vector has fewer than two items.
 * 
 * # Panics
 * - In non-release builds, constructing a Merkle Tree will panic if we call the constructor
 * with a vector of fewer than two elements.
 * 
 * # Examples
 * 
 * ```
 * let data = vec!("some", "sample", "data");
 * let mrkl_tree = merkle::MerkleTree::construct(&mut data);
 * match mrkl_tree.validate() {
 *     merkle::MrklVR::Valid => {}
 *     _ => assert!(false)
 * }
 * 
 * ```
 *  
 */

use hash::{Hashable, HashPointer};
use self::{
    MrklVR::*,
    MerkleBranch::*
};

/**
 * An enumerations of children types for `MerkleTree`.
 * ---
 * When a child contains another `MerkleTree`, it is specified as `MerkleBranch::Branch`.
 * 
 * When a child is a leaf, it is specified as `MerkleBranch::Leaf`. Leaves contain 
 * an object of type `T` and a `String` which is the sha2 hash of that object.  
 * 
 * If a child is `MerkleBranch::Partial`, we are dealing with a pruned tree. 
 * `MerkleTree::validate` will never return `Valid` for a Merkle tree with 
 * `Partial` branches, for that you must use `MerkleTree::validate_pruned`. 
 * 
 * A child can also be `MerkleBranch::None`, if it contains no information.
 */
enum MerkleBranch {
    Branch(Box<MerkleTree>),
    Leaf(String),
    Empty
}

/**
 * A struct representing a Merkle Tree, which may or may not be an internal node.
 * 
 * # Fields
 * `left`: The left child of the `MerkleTree`, held within a `MerkleBranch` enumeration.
 * 
 * `right`: The right child of the `MerkleTree`, held within a `MerkleBranch` enumeration.
 * 
 * `l_bound`: The largest element in the Merkle tree who has `left` as an ancestor
 * 
 * `r_bound`: The largest element in the Merkle tree who has `right` as an ancestor
 * 
 * `mrkl_root`: The hash of each of this node's children -- sha2(left.mrkl_root || right.mrkl_root).
 * 
 * `height`: The height of the current node in the overall `MerkleTree`. Leaves have height 0.
 */
pub struct MerkleTree {
    
    left: MerkleBranch,
    right: MerkleBranch,

    mrkl_root: String,
    
    height: usize 
}

/**
 * The Merkle Validation Result enumerates the possible results of calling
 * `MerkleTree::validate` on a Merkle tree.
 * 
 * The result is `Valid` if there are no inconsistencies when validating the tree.
 * 
 * `InvalidHash` represents a situation when the hash of the children of a `MerkleTree`
 * do not equal the tree's `mrkl_root`. 
 * 
 * `InvalidTree` represents a situation where the given `MerkleTree` is malformed. For example,
 * its left child is a leaf and its right child is a branch.
 * 
 * `InvalidHash` and `InvalidTree` will both contain a `String` which gives more information
 * on how the validation failed.
 */
pub enum MrklVR {
    Valid,
    InvalidHash(String), //String values contain an error message with a description
    InvalidTree(String)  //of what went wrong
}

impl MerkleTree {


    /**
     * Constructs a `MerkleTree` instance.
     * 
     * # Arguments
     * - `data`: A vector of data which will be used to build the `MerkleTree` instance. For example, if data
     * was `vec!(x, y, z)`, then the resulting `MerkleTree` would be
     * 
     *           h(h(h(x)||h(y))||h(h(z)))
     *               /        \
     *              /          \ 
     *        h(h(x)||h(y))    h(h(z))
     *           /   \          |
     *          /     \         |
     *         /       \        |
     *       h(x)     h(y)     h(z) 
     *        |        |        | 
     *        x        y        z
     * 
     * # Panics
     * In non-release builds, will panic if `data.len()` is less than 2.
     * 
     * # Errors
     * May return an error if it fails to construct leaves correctly.
     * Will return an error result if the length of `data` is less than 2. 
     */
    pub fn construct<T: Hashable>(mut data: Vec<T>) -> Result<Self, String> {

        if data.len() < 1 {
            debug_assert!(false, "Wrong number of arguments to merkle tree constructor.");

            return Err(String::from(
                "Not enough data to construct Merkle Tree. Must receive at least two items."
            ));
        }

        let mut mrkl_trees: Vec<MerkleTree> = Vec::new();

        while data.len() > 0 {

            let fringe_node = MerkleTree::construct_fringe_node(&mut data);
            match fringe_node {
                Ok(node) => mrkl_trees.push(node),
                Err(msg) => { return Err(msg); }
            }

        }

        let mut height = 1;

        while mrkl_trees.len() > 1 {

            let mut new_mrkl_trees: Vec<MerkleTree> = Vec::new();

            while mrkl_trees.len() > 0 {

                let internal_node = MerkleTree::construct_internal_node(&mut mrkl_trees, height);
                match internal_node {
                    Ok(node) => new_mrkl_trees.push(node),
                    Err(msg) => { return Err(msg); }
                }
                
            }

            mrkl_trees = new_mrkl_trees;
            height += 1;        
        }
        Ok(mrkl_trees.remove(0))
    }


    /**
     * Reports whether or not a given item is contained within one of the leaves of the Merkle tree.
     * The merkle leaves are sorted, so this method binary searches for the correct leaf in O(log n) time.
     * 
     * # Arguments
     * `item`: A borrow of the item you want to search for
     * 
     * # Return Value
     * Returns `true` if it finds a leaf in the merkle tree with data equal to `item`, and `false` otherwise. 
     * 
     * # Errors
     * Searching for an item in a pruned tree will only work if the item was not pruned. Otherwise,
     * There is usually no way to tell whether or not that item was ever in the tree before it was pruned.
     * Therefore, if during the exectution of `contains` the search encounters a partial branch, it will
     * return an error.
     */
    pub fn contains(&self, item_hash: &str) -> bool {
        
        let mut result = false;
        match &self.left {
            Branch(node) => result = node.contains(item_hash),
            Leaf(hash) => result = hash == item_hash,
            _ => {}
        }
        match &self.right {
            Branch(node) => result = result || node.contains(item_hash),
            Leaf(hash) => result = result || hash == item_hash,
            _ => result = result || false
        }

        result
    } 

    /**
     * Validates a given instance of `MerkleTree`.
     * 
     * # Return Value
     * Returns a `MrklVR` enumeration. See the documentation for `MrklVR` for the meanings
     * of each result.
     * 
     * *Note*: This method will return InvalidTree if called on a pruned `MerkleTree` instance.
     * Use `MerkleTree::validate_pruned` in those cases which validation of a pruned Merkle tree
     * is required.
     * 
     * # Panics
     * In non-release builds panics if, when validating a fringe node, it encounters a situation
     * where a right item hash is given but no right item is given, or vice versa. Note that in 
     * release builds this will cause `validate` to return `MrklVR::InvalidHash`.
     */
    fn validate(&self) -> MrklVR {
       
        //##################################################################
        //TODO: make sure leaves are in order.

        match (&self.left, &self.right) {
           
           /*
           * If there are two branches, then we recursively validate each branch.
           * If they are both valid, then we return the result of self.validate_internal_node.
           * Otherwise, we propagate whichever Invalid result was returned by calling validate
           * on each branch.
           */
           (Branch(ref left_br), Branch(ref right_br)) => {
               
                match (left_br.validate(), right_br.validate()) {
                    
                    (Valid, Valid) => self.validate_internal_node(&left_br, Some(&right_br)),

                    (result@InvalidHash(_), _) | (_, result@InvalidHash(_)) => result,

                    (result@_,_) => result,
                }
            }

            /*
            * If the right branch is empty and the left is a branch, then we validate the
            * left branch only. We call self.validate_internal_node with Option::None as the right
            * branch if the left branch passes the validation.
            */
            (Branch(ref branch), Empty) => {

                match branch.validate() {
                    Valid => self.validate_internal_node(branch, None),
                    result@InvalidHash(_) | result@InvalidTree(_) => result
                }
                
            }

            /*
            * If both children are leaves, then we can simply call self.validate_fringe_node.
            * We no longer have to worry about recursively calling validate in this case since
            * leaves just contain raw objects.
            */
            (Leaf(ref left_hash), Leaf(ref right_hash)) 
                    => self.validate_fringe_node(left_hash, Some(right_hash)),
            
            /*
            * If the left child is a leaf and the right is empty, we pass in the Option::None 
            * argument to self.validate_fringe_node accordingly. Note that we must pass in 
            * None to both right_it and right_hash, since it would not make sense to have
            * one without the other. An invalid result will always be returned if we do not
            * do so.
            */
            (Leaf(ref hash), Empty) 
                    => self.validate_fringe_node(hash, None),

            /*
            * Any other pattern for the children of a Merkle node would imply some sort of
            * error in the structure of the tree. Therefore, we always report that we have a malformed tree
            * if we get this far.
            */        
            (_,_) => InvalidTree(String::from("Malformed tree"))
        }
    }


    /**
     * Helper function for `MerkleTree::Validate` which validates an internal node in the Merkle tree.
     * It first computes the concatenated hash for its two children, and compares that with its
     * `mrkl_root`. It then checks that the height of its children are one less than its height.
     * 
     * If `right_node` is `Option::None`, then the function will proceed accordingly by treating
     * the `MerkleTree` as a node with a single child.
     */
    fn validate_internal_node(&self, left_node: &MerkleTree, right_node: Option<&MerkleTree>) -> MrklVR {

        let mut hash = String::new();
        hash.push_str(&left_node.mrkl_root);

        let mut right_has_correct_height = true;
        match right_node {

            Some(r) => {
                hash.push_str(&r.mrkl_root);

                right_has_correct_height = self.height == r.height + 1;
            }

            None => {}
        }

        hash = hash.get_hash();
    
        if hash == self.mrkl_root && 
           self.height == left_node.height + 1 &&
           right_has_correct_height
        { 
               Valid 
        }
        else if self.height != left_node.height + 1 ||
                !right_has_correct_height
        {
            InvalidTree(String::from("An internal node has height which differs from 1 + (child height)"))
        } 
        else { 
            InvalidHash(String::from("An internal node has an unexpected mrkl_root"))
        }
    }

    /**
     * Helper function for `MerkleTree::Validate` which validates a fringe node in the Merkle tree.
     * It first computes the concatenated hash for its children, and compares that with its
     * `mrkl_root`. It then checks that its height is 0.
     */
    fn validate_fringe_node(&self, left_hash: &str, right_hash: Option<&str>)
            -> MrklVR {
        
        let mut hash  = String::new();
        hash.push_str(left_hash);

        match right_hash {
            Some(r) => {
                hash.push_str(r);
            }
            None => {}
        }    

        hash = hash.get_hash();

        
        if  self.mrkl_root == hash && self.height == 0 {  
            Valid

        } else if self.mrkl_root != hash {
            InvalidHash(String::from("A fringe node has an unexpected mrkl_root"))
        }
        else {
            InvalidTree(String::from("A fringe node has nonzero height"))
        } 
    }

    /*
    --------------------------------------------------------------------------------------------------------
    |                                    Private construct methods                                         |
    --------------------------------------------------------------------------------------------------------
    */

    /**
     * Helper function for `MerkleTree::construct`. Pops off the first element of 
     * `data` and creates a `MerkleBranch::Leaf`. It also pushes the hash of this first element
     * into `hash`.
     */
    fn construct_leaf<T: Hashable>(data: &mut Vec<T>, hash: &mut String) -> MerkleBranch {
            
            let first = data.remove(0);
            let first_hash = first.get_hash();
            
            hash.push_str(&first_hash);

            Leaf(first.get_hash())
    }

    /**
     * Helper function for `MerkleTree::construct`. Pops off the first element of `data`
     * and creates a `MerkleBranch::Branch`. Also pushes the hash of this first element
     * onto `hash`.
     */
    fn construct_branch(data: &mut Vec<MerkleTree>, hash: &mut String) -> MerkleBranch {
        
        let first = data.remove(0);
        hash.push_str(&first.mrkl_root);

        Branch(Box::new(first))
    }

    /**
     * Helper function for `MerkleTree::construct`. Creates a `MerkleTree` from the 
     * first two elements of `data`, where the children of this `MerkleTree` are
     * leaves.
     */
    fn construct_fringe_node<T: Hashable>(data: &mut Vec<T>) -> Result<MerkleTree, String> {    
       
        let mut hash = String::new();

        let left_leaf = MerkleTree::construct_leaf(data, &mut hash);

        let mut right_leaf = Empty;
        if data.len() > 0 {
            
            right_leaf = MerkleTree::construct_leaf(data, &mut hash);
            
        }
        hash = hash.get_hash();

        Ok(MerkleTree{
            left: left_leaf,
            right: right_leaf,
            mrkl_root: hash,
            height: 0
        })
    }

    /**
     * Helper function for `MerkleTree::construct`. Creates a `MerkleTree` from the first
     * two elements of `data`, where the children of this `MerkleTree` are other `MerkleTree`s. 
     */
    fn construct_internal_node(data: &mut Vec<MerkleTree>, height: usize) -> Result<MerkleTree, String> {
        let mut hash = String::new();

        let left_branch = MerkleTree::construct_branch(data, &mut hash);

        let mut right_branch = Empty;
        if data.len() > 0 {
            right_branch = MerkleTree::construct_branch(data, &mut hash);
               
        }

        hash = hash.get_hash();

        Ok(MerkleTree {
            left: left_branch,
            right: right_branch,
            mrkl_root: hash,
            height
        })
    }
}