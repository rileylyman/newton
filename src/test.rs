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
    for i in (1..10000).step_by(2) {
        v.push(i.to_string());
    }
    let mut m_tree = merkle::MerkleTree::construct(v).unwrap();

    for i in (1..10000).step_by(2) {
        assert!(m_tree.contains(&i.to_string()).unwrap());
    }
    for i in (2..10000).step_by(2) {
        assert!(!m_tree.contains(&i.to_string()).unwrap());     
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

    /*These checks work*/
    let eleven = 11.to_string();
    let one01 = 101.to_string();
    assert!(m_tree.contains(&eleven).unwrap());
    assert!(m_tree.contains(&one01).unwrap());

    /*These checks also work.*/
    let to_check = vec!(11.to_string(), 101.to_string());
    for element in to_check {
        assert!(m_tree.contains(&element).unwrap());
    }

    /*But pruning still returns false because to_check is not contained in the tree.*/
    if m_tree.prune(&to_check) {
        match m_tree.validate() {
            merkle::MrklVR::InvalidTree(_) => {}
            _ => assert!(false) 
        }
    } else {
        assert!(false);
    }

}

#[test]
fn merkle_contains() {
    let m_tree = merkle::MerkleTree::construct(vec!(1.to_string(), 3.to_string())).unwrap();
    
    assert!(!m_tree.contains(&2.to_string()).unwrap())
}
