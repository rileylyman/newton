# Newton

## Overview
Implementations of common data structures and algorithms used in the creation of Blockchain applications. 

### Currently supported implementations:
- Merkle Trees
- Hash Pointers and Blockchains

## Merkle Trees
The implementation of the Merkle Tree data structure can be found [here](https://github.com/rileylyman/newton/tree/master/src/merkle.rs). A `MerkleTree<T>` instance enforces the trait bounds `T: Hashable + Ord + Clone`. Note that `String` alreay has an implementation of `Hashable` defined in hash.rs. As long as you can convert `T` to a `String` representation, you can easily implement `Hashable`.

### Supported Methods
- Construction: The `MerkleTree<T>::construct` method takes a vector of `T` you would like to place into a `MerkleTree`, and returns a new `MerkleTree<T>` instance. Example:
```
let names = vec!(String::from("first"),
        String::from("second"),
        String::from("third"),
        String::from("fourth"),
        String::from("fifth")
    );
let mrkl_tree = merkle::MerkleTree::construct(names).unwrap();
```
- Containment checking: The `MerkleTree<T>::contains` method takes an `&T` borrow and checks in `O(log n)` with binary search time whether or not the tree contains that element. `contains` returns `Result<bool, String>`. Example:
```
assert!(mrkl_tree.contains(&String::from("first)).unwrap());
assert!(!mrkl_tree.contains(&String::from("tenth")).unwrap());
```
- Pruning: The `MerkleTree<T>::prune` method takes a slice `&[T]` and destructively prunes the Merkle tree by removing any leaf nodes that are not contained in the slice.

- Validation: The `MerkleTree<T>::validate` method validates a non-pruned tree, and `MerkleTree<T>::validate_pruned` validates pruned trees. Both methods return a Merkle Validation Result (`MrklVR`) enumeration, which can be `Valid`, `InvalidHash`, or `InvalidTree`. 

## Hash Pointers and Blockchains
The implementations of both the Hash Pointer and Blockchain data structures can be found [here](https://github.com/rileylyman/newton/tree/master/src/hash.rs). A `HashPointer<T>` instance contains a boxed reference to some instance of `T` along with the objects hash. Therefore, we have the trait bound `T: Hashable`.
