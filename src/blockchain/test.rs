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
    
    assert!(mrkl_tree.contains(&String::from("alice")).unwrap());
    assert!(!mrkl_tree.contains(&String::from("mje")).unwrap());

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

#[test]
fn merkle2() {
    let mut v = Vec::new();
    for i in 1..10000 {
        v.push(i.to_string());
    }
    let m_tree = merkle::MerkleTree::construct(v).unwrap();

    for i in 1..10000 {
        assert!(m_tree.contains(&i.to_string()).unwrap());
    }
    for i in 10000..20000 {
        assert_eq!(m_tree.contains(&i.to_string()).unwrap(), false);
    }

    match m_tree.validate() {
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

    match m_tree.validate() {
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
