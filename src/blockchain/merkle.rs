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
enum MerkleBranch<T : Hashable + Ord + Clone> {
    Branch(Box<MerkleTree<T>>),
    Leaf(T, String),
    Partial(String),
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
pub struct MerkleTree<T : Hashable + Ord + Clone> {
    
    left: MerkleBranch<T>,
    right: MerkleBranch<T>,
    
    l_bound: T, //TODO: Make this option and delete it for pruned nodes
    r_bound: T,

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

impl<T: Hashable + Ord + Clone> MerkleTree<T> {


    /**
     * Constructs a `MerkleTree` instance.
     * 
     * # Arguments
     * - `data`: A vector of data which will be used to build the `MerkleTree` instance. For example, if data
     * was `vec!(x, y, z)`, then the resulting `MerkleTree` would be
     * 
     *     h(h(h(x)||h(y))||h(h(z)))
     *         /        \
     *        /          \ 
     *  h(h(x)||h(y))    h(h(z))
     *     /   \          |
     *    /     \         |
     *   /       \        |
     * h(x)     h(y)     h(z) 
     *  |        |        | 
     *  x        y        z
     * 
     * # Panics
     * In non-release builds, will panic if `data.len()` is less than 2.
     * 
     * # Errors
     * May return an error if it fails to construct leaves correctly.
     * Will return an error result if the length of `data` is less than 2. 
     */
    pub fn construct(mut data: Vec<T>) -> Result<Self, String> {

        data.sort();

        if data.len() < 1 {
            debug_assert!(false, "Wrong number of arguments to merkle tree constructor.");

            return Err(String::from(
                "Not enough data to construct Merkle Tree. Must receive at least two items."
            ));
        }

        let mut mrkl_trees: Vec<MerkleTree<T>> = Vec::new();

        while data.len() > 0 {

            let fringe_node = MerkleTree::construct_fringe_node(&mut data);
            match fringe_node {
                Ok(node) => mrkl_trees.push(node),
                Err(msg) => { return Err(msg); }
            }

        }

        let mut height = 1;

        while mrkl_trees.len() > 1 {

            let mut new_mrkl_trees: Vec<MerkleTree<T>> = Vec::new();

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
     * A destructive method which prunes a Merkle tree, only keeping branches which
     * lead to the elements specified in `to_keep`. Unnecessary branches are converted 
     * to `MerkleBranch::Partial(hash)`, where hash is the value of the `mrkl_root` of
     * the node that was pruned. 
     * 
     * *Note*: After a Merkle tree has been pruned, you must use the method `validate_pruned` 
     * instad of `validate` to check if the tree is valid.
     * 
     * # Arguments
     * `to_keep`: An array slice which lists the leaves you wish to keep in the Merkle tree.
     * 
     * # Return Value
     * Returns `true` if there were no errors during pruning, and `false` otherwise. 
     * 
     * # Examples
     *  
     * Consider the following scenario:
     * 
     * Calling `prune` on the left tree with `to_keep=[y]` yields the tree on the right.
     *         
     *   
     *            h3                             h3
     *           /  \                           /  \
     *          /    \                         /    \
     *         /      \                       /      \
     *        /        \                     /        \
     *       /          \     -->   -->     /          \
     *      /            \                 /            \
     *     h1            h2               h1            h2  
     *    /  \          /  \             /  \          
     *   /    \        /    \           /    \            
     *  /      \      /      \         /      \      
     * hx      hy    hz       hw     hx       hy           
     * |       |     |        |                |
     * x       y     z        w                y
     * 
     * 
     * In the resulting tree, the right child of `root` and the left child of `h1` are now just hashes.
     *
     * # Errors
     * - Will return false if `to_keep` is empty, since this would be effectively pruning the
     * entire tree away. 
     * - There are a number of errors that could occur when pruning malformed trees, so it may be advisable
     * to validate a tree before pruning, unless you are certain the tree is valid. 
     * 
     */
    pub fn prune(&mut self, to_keep: &[T]) -> bool {
        
        // The tree we are pruning must be valid. Otherwise there is 
        // no way for us to check whether all the elements in `to_keep`
        // are contained within the tree, and therefore no way for us to 
        // recurse properly. All the elements of the tree must be sorted as
        // well, which is also verifed by validate. TODO
        
        if let Valid = self.validate() {} else { // Check if tree is valid
            return false;
        }

        let mut result = true;
        if to_keep.len() <= 0 { return false; } // Cannot prune an entire tree

        //TODO: Make sure all elements in to_keep are in tree

        let mut prune_left = true;
        for element in to_keep {
            if *element <= self.l_bound { prune_left = false; }
        }

        result = result && MerkleTree::prune_recurse(to_keep, &mut self.left, prune_left);

        let mut prune_right = true; {           // We start a new scope here since self.find_min_right()
                                                // borrows self.right
            let min_right;                      // We use the reference to one of the leaves of 
            match self.find_min_right() {       // the tree to compute whether or not all the elements
                Ok(x) => { min_right = x; }     // of to_keep are less than the min_right value,
                _ => { return result; }         // but after that we stop borrowing immutably so we can
            }                                   // mutably borrow when mutating the right branch.

            for element in to_keep {
                if *element >= *min_right {
                    prune_right = false;
                }
            }
        }

        result && MerkleTree::prune_recurse(to_keep, &mut self.right, prune_right)
    }


    fn prune_recurse(to_keep: &[T], branch: &mut MerkleBranch<T>, should_prune: bool) -> bool {
        
        let compute_branch = |br: &mut MerkleBranch<T>| {
            match br {
                Branch(node) =>  { Ok(Partial(node.mrkl_root.clone())) }
                Leaf(_, hash) => { Ok(Partial(hash.clone())) }
                Partial(hash) => { Ok(Partial(hash.clone())) }
                _ => Err(String::from("Cannot prune empty branch"))
            }
        };
        
        if should_prune {
            match compute_branch(branch) {
                Ok(pruned) => { *branch = pruned; true }
                _ => { return false; }
            }
        } else {
            match branch {
                Branch(node) => { node.prune(to_keep) }
                _ => { true }
            }
        }
    }

    /**
     * Finds the leftmost leaf in the right child of the given Merkle tree. This will 
     * be the minimum value to the right of the current Merkle root if the tree is sorted.
     * 
     * #Errors
     * Will return an error if the right branch is partial or empty.  
     */
    fn find_min_right(&self) -> Result<&T, String> {
        match &self.right {
            Branch(node) => node.find_min(),
            Leaf(value,_) => Ok(value),
            _ => Err(String::from("There is nowhere to search to the right to find the minimum element"))
        }
    }

    /**
     * Finds the leftmost leaf value in a given Merkle Tree. This will be the 
     * minimum value if the tree is sorted. 
     * 
     * # Errors
     * Will return an error if the left branch is partial or empty.
     */
    fn find_min(&self) -> Result<&T, String> {
        match &self.left {
            Branch(node) => node.find_min(),
            Leaf(value, _) => Ok(value),
            _ => Err(String::from("Couldn't go left anymore when finding minimum element"))
        }
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
    pub fn contains(&self, item: &T) -> Result<bool, String> {
        let search_branch = if *item <= self.l_bound {
            &self.left
        } else {
            &self.right
        };
        
        match search_branch {
            Branch(node) => node.contains(item),
            Leaf(value, _) => Ok(*value == *item),
            Partial(_) => Err(String::from("Could not search further in pruned tree")),
            _ => Ok(false)
        }
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
    pub fn validate(&self) -> MrklVR { 
        self._validate(false)
    }

     /**
     * Validates a given pruned instance of `MerkleTree`.
     * 
     * # Return Value
     * Returns a `MrklVR` enumeration. See the documentation for `MrklVR` for the meanings
     * of each result.
     * 
     * # Panics
     * In non-release builds panics if, when validating a fringe node, it encounters a situation
     * where a right item hash is given but no right item is given, or vice versa. Note that in 
     * release builds this will cause `validate` to return `MrklVR::InvalidHash`.
     */
    pub fn validate_pruned(&self) -> MrklVR {
        self._validate(true)
    }


    /*
    --------------------------------------------------------------------------------------------------------
    |                                   Private MerkleTree methods below                                   |
    --------------------------------------------------------------------------------------------------------
    */

    /**
     * Function which drives the validation of a Merkle tree. If pruned is false, then
     * it will call any tree invalid with pruned hashes.  
     */
    fn _validate(&self, pruned: bool) -> MrklVR {
       
        match (&self.left, &self.right) {
           
           /*
           * If there are two branches, then we recursively validate each branch.
           * If they are both valid, then we return the result of self.validate_internal_node.
           * Otherwise, we propagate whichever Invalid result was returned by calling validate
           * on each branch.
           */
           (Branch(ref left_br), Branch(ref right_br)) => {
               
                match (left_br._validate(pruned), right_br._validate(pruned)) {
                    
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

                match branch._validate(pruned) {
                    Valid => self.validate_internal_node(branch, None),
                    result@InvalidHash(_) | result@InvalidTree(_) => result
                }
                
            }

            /*
            * If both children are leaves, then we can simply call self.validate_fringe_node.
            * We no longer have to worry about recursively calling validate in this case since
            * leaves just contain raw objects.
            */
            (Leaf(ref left_it, ref left_hash), Leaf(ref right_it, ref right_hash)) 
                    => self.validate_fringe_node(left_it, left_hash, Some(right_it), Some(right_hash)),
            
            /*
            * If the left child is a leaf and the right is empty, we pass in the Option::None 
            * argument to self.validate_fringe_node accordingly. Note that we must pass in 
            * None to both right_it and right_hash, since it would not make sense to have
            * one without the other. An invalid result will always be returned if we do not
            * do so.
            */
            (Leaf(ref left_it, ref left_hash), Empty) 
                    => self.validate_fringe_node(left_it, left_hash, None, None),

            /*
            * If both children are partial, then we have no information to go off of. 
            * We have no choice but to return an InvalidTree specification.
            */
            (Partial(_),Partial(_)) 
                    => InvalidTree(String::from("Invalid pruned tree. Only one child may be pruned.")),

            /*
            * Otherwise, if only one child is partial, then we can call self.evaluate_pruned_node.
            */
            (Partial(hash), other@_) | (other@_, Partial(hash)) => {
                if !pruned { InvalidTree(String::from("Unexpected pruned tree.")) }
                else {
                    self.validate_pruned_node(hash, other)
                }
            }

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
    fn validate_internal_node(&self, left_node: &MerkleTree<T>, right_node: Option<&MerkleTree<T>>) -> MrklVR {

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
                right_has_correct_height
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
    fn validate_fringe_node(&self, left_it: &T, left_hash: &str, right_it: Option<&T>, right_hash: Option<&str>)
            -> MrklVR {
        
        let mut hash  = String::new();
        hash.push_str( left_hash);

        let mut right_hash_is_valid = true;
        match (right_it, right_hash) {

            (Some(r), Some(r_hash)) => {
                hash.push_str(&r_hash);

                right_hash_is_valid = r.get_hash() == r_hash;
            }

            (None, None) => {}

            (_,_) => {
                debug_assert!(false, 
                    "Upon validating a fringe node, expected both right_it and right_hash to be None"
                );
                return InvalidTree(String::from(
                    "Upon validating a fringe node, expected both right_it and right_hash to be None"
                ));
            }
        }    

        hash = hash.get_hash();

        
        if  left_it.get_hash() == *left_hash && 
            right_hash_is_valid &&
            self.mrkl_root == hash &&
            self.height == 0 {
            
            Valid
        } else if self.mrkl_root != hash {
            InvalidHash(String::from("A fringe node has an unexpected mrkl_root"))
        }
        else if self.height != 0 {
            InvalidTree(String::from("A fringe node has nonzero height"))
        } else {
            InvalidHash(String::from("A leaf's hash failed a hash check"))
        }
    }

    /**
     * Helper function for `MerkleTree::Validate` which validates a  node in the Merkle tree
     * which has a partial child. It enumerates the other child. If the other child is a branch,
     * then the branches hash concatenated with the pruned hash must hash to this node's mrkl_root.
     * If the branch is a leaf, a similar check occurs, and we must further check that the leaf's 
     * item hash still matches the computed item hash. In any other case we propagate Invalid errors.
     */
    fn validate_pruned_node(&self, pruned_hash: &str, other: &MerkleBranch<T>) -> MrklVR {
        match other {
            Branch(node) => {
                match node.validate() {
                    Valid => {
                        let mut hash = String::new();
                        hash.push_str(pruned_hash);
                        hash.push_str(&node.mrkl_root);
                        hash = hash.get_hash();
                        if self.mrkl_root == hash {
                            Valid
                        } else {
                            InvalidHash(String::from("An internal node had an unexpected mrkl_root"))
                        }
                    } 
                    result@_ => result
                }  
            }
            Leaf(ref item, ref item_hash) => {
                let mut hash = String::new();
                hash.push_str(item_hash);
                hash.push_str(pruned_hash);
                hash = hash.get_hash();
                if item_hash == &item.get_hash() && hash == self.mrkl_root {
                    Valid
                } else if item_hash != &item.get_hash() {
                    InvalidHash(String::from("A leaf's hash failed a hash check"))
                } else {
                    InvalidHash(String::from("A fringe node has an unexpected mrkl_root"))
                }
            }
            Partial(_) => InvalidTree(String::from("Invalid pruned tree. Only one child may be pruned.")),
            Empty => InvalidTree(String::from("Invalid pruned tree. Every node must
                     have at least one valid child. This node has one empty and one partial child.")),
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

            Leaf(first, first_hash)
    }

    /**
     * Helper function for `MerkleTree::construct`. Pops off the first element of `data`
     * and creates a `MerkleBranch::Branch`. Also pushes the hash of this first element
     * onto `hash`.
     */
    fn construct_branch(data: &mut Vec<MerkleTree<T>>, hash: &mut String) -> MerkleBranch<T> {
        
        let first = data.remove(0);
        hash.push_str(&first.mrkl_root);

        Branch(Box::new(first))
    }

    /**
     * Helper function for `MerkleTree::construct`. Creates a `MerkleTree` from the 
     * first two elements of `data`, where the children of this `MerkleTree` are
     * leaves.
     */
    fn construct_fringe_node(data: &mut Vec<T>) -> Result<MerkleTree<T>, String> {    
       
        let mut hash = String::new();

        let left_leaf = MerkleTree::construct_leaf(data, &mut hash);

        let mut right_leaf = Empty;
        if data.len() > 0 {
            
            right_leaf = MerkleTree::construct_leaf(data, &mut hash);
            
        }
        hash = hash.get_hash();

        let l_bound;
        match left_leaf {
            Leaf(ref value,_) => { l_bound = value.clone(); }
            _ => { return Err(String::from("Leaf contains no data")); }
        }

        let mut r_bound = l_bound.clone();
        match right_leaf {
            Leaf(ref value,_) => { r_bound = value.clone(); }
            _ => {}
        }

        Ok(MerkleTree{
            left: left_leaf,
            right: right_leaf,
            l_bound,
            r_bound,
            mrkl_root: hash,
            height: 0
        })
    }

    /**
     * Helper function for `MerkleTree::construct`. Creates a `MerkleTree` from the first
     * two elements of `data`, where the children of this `MerkleTree` are other `MerkleTree`s. 
     */
    fn construct_internal_node(data: &mut Vec<MerkleTree<T>>, height: usize) -> Result<MerkleTree<T>, String> {
        let mut hash = String::new();

        let left_branch = MerkleTree::construct_branch(data, &mut hash);

        let mut right_branch = Empty;
        if data.len() > 0 {
            right_branch = MerkleTree::construct_branch(data, &mut hash);
               
        }

        hash = hash.get_hash();

        let l_bound;
        match left_branch {
            Branch(ref node) => { l_bound = node.r_bound.clone(); }
            _ => { return Err(String::from("There was no r_bound to clone")); }
        }

        let mut r_bound = l_bound.clone();
        match right_branch {
            Branch(ref node) => { r_bound = node.r_bound.clone(); }
            _ => {}
        }

        Ok(MerkleTree {
            left: left_branch,
            right: right_branch,
            l_bound,
            r_bound,
            mrkl_root: hash,
            height
        })
    }
}