use super::*;
use hash::Hashable;

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
    
    assert!(mrkl_tree.contains_item(&String::from("alice")));
    assert!(!mrkl_tree.contains_item(&String::from("mje")));

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
    let m_tree = merkle::MerkleTree::construct(v).unwrap();

    for i in (1..10000).step_by(2) {
        assert!(m_tree.contains_item(&i.to_string()));
    }
    for i in (2..10000).step_by(2) {
        assert!(!m_tree.contains_item(&i.to_string()));     
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

#[test]
fn merkle_proof() {
    let mut v = Vec::new();
    for i in (1..10000).step_by(2) {
        v.push(i.to_string());
    }
    let m_tree = merkle::MerkleTree::construct(v).unwrap();

    let m_proof = m_tree.gen_proof(&107.to_string());
    match m_proof {
        Some(proof) => {
            assert!(proof.check_proof_form(m_tree.get_mrkl_root(), m_tree.get_height()));
            assert!(proof.verify(107.to_string()));
        }
        _ => assert!(false)
    }
    let m_proof = m_tree.gen_proof(108.to_string());
    match m_proof {
        Some(proof) => assert!(false),
        _ => assert!(true)
    }

}

#[test]
fn merkle_contains() {
    let m_tree = merkle::MerkleTree::construct(vec!(1.to_string(), 3.to_string())).unwrap();
    
    assert!(!m_tree.contains_item(&2.to_string()));
}
