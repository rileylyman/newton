use super::*;

#[test]
fn hash_pointer() {
    let name = String::from("riley");
    let hash_ptr = HashPointer::to(name);
    print!("Name:  {}, with hash of: {}\n", hash_ptr.ptr, hash_ptr.hash);
}

#[test]
fn merkle1() {
    let names = vec!(String::from("sally"),
        String::from("alice"),
        String::from("ronnie"),
        String::from("mj"),
        String::from("john john")
    );
    let mrkl_tree = merkle::MerkleTree::construct(names).unwrap();
    
    match mrkl_tree.validate() {
        merkle::MrklVR::Valid => {
            println!("Valid");
            assert!(true);
        }
        merkle::MrklVR::InvalidHash(x) => {
            println!("Invalid Hash: {}", x);
            assert!(false);
        }
        merkle::MrklVR::InvalidTree(x) => {
            println!("Invalid Tree: {}", x);
            assert!(false);
        }
    }
}
