use super::*;

#[test]
fn hash_pointer() {
    let name = String::from("riley");
    let hash_ptr = hash::HashPointer::to(name);
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
    let mut m_tree = merkle::MerkleTree::construct(v).unwrap();

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

    if m_tree.prune(&[10.to_string(), 100.to_string()]) {
        match m_tree.validate() {
            merkle::MrklVR::InvalidTree(_) => { println!("correct invalud"); assert!(true)}
            _ => assert!(false)
        }
    } else {
        assert!(false);
    }

    match m_tree.validate_pruned() {
        merkle::MrklVR::Valid => { println!("pruned valid"); assert!(true) } 
        _ => assert!(false)
    }

    assert!(m_tree.contains(&10.to_string()).unwrap());
    match m_tree.contains(&132.to_string()) {
        Err(_) => assert!(true),
        Ok(x) => {println!("{}", x); assert!(false)}
    }

}
