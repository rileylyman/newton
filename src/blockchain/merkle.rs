use super::Hashable;

enum MerkleBranch<T : Hashable> {
    Branch(Box<MerkleTree<T>>),
    Leaf(T, String),
    None
}

pub struct MerkleTree<T : Hashable> {
    left: MerkleBranch<T>,
    right: MerkleBranch<T>,
    mrkl_root: String,
    depth: usize
}

pub enum MrklVR { //Merkle Validation Result
    Valid,
    InvalidHash,
    InvalidTree
}

impl<T: Hashable> MerkleTree<T> {
    pub fn construct(data: &mut Vec<T>) -> Self {
        assert!(data.len() > 1);

        let mut mrkl_trees: Vec<MerkleTree<T>> = Vec::new();
        
        let length = data.len();
        for _ in (0..length/2*2).step_by(2) {

            let first = data.remove(0);
            let first_hash = first.get_hash();
            let second = data.remove(0);
            let second_hash = first.get_hash();

            let mut combined_hash = String::new();
            combined_hash.push_str(&first_hash);
            combined_hash.push_str(&second_hash);
            combined_hash = combined_hash.get_hash();

            mrkl_trees.push(MerkleTree{
                left: MerkleBranch::Leaf(first, first_hash),
                right: MerkleBranch::Leaf(second, second_hash),
                mrkl_root: combined_hash,
                depth: 0
            });
        }

        for _ in length/2*2..length {

            let first = data.remove(0);
            let first_hash = first.get_hash();
            let combined_hash = first.get_hash();

            mrkl_trees.push(MerkleTree{
                left: MerkleBranch::Leaf(first, first_hash),
                right: MerkleBranch::None,
                mrkl_root: combined_hash,
                depth: 0
            });
        }

        let mut depth = 1;

        while mrkl_trees.len() > 1 {

            let length = mrkl_trees.len();
            for _ in (0..(length / 2 * 2)).step_by(2) {
                let left = Box::new(mrkl_trees.remove(0));
                let right = Box::new(mrkl_trees.remove(0));

                let mut combined_hash = String::new();
                combined_hash.push_str(&left.mrkl_root);
                combined_hash.push_str(&right.mrkl_root);
                combined_hash = combined_hash.get_hash();

                mrkl_trees.push(MerkleTree{
                    left: MerkleBranch::Branch(left),
                    right: MerkleBranch::Branch(right),
                    mrkl_root: combined_hash,
                    depth
                });
            }

            for _ in length/2*2..length {
                let left = Box::new(mrkl_trees.remove(0));
                let mut combined_hash = String::new();
                combined_hash.push_str(&left.mrkl_root);
                combined_hash = combined_hash.get_hash();
                mrkl_trees.push(MerkleTree{
                    left: MerkleBranch::Branch(left),
                    right: MerkleBranch::None,
                    mrkl_root: combined_hash,
                    depth
                });
            }
            depth += 1;        
        }
        mrkl_trees.remove(0)
    }

    pub fn validate(&self) -> MrklVR {
        match (&self.left, &self.right) {
           (MerkleBranch::Branch(left_br), MerkleBranch::Branch(right_br)) => {
                match (left_br.validate(), right_br.validate()) {
                    (MrklVR::Valid, MrklVR::Valid) => MrklVR::Valid,
                    (MrklVR::InvalidHash, MrklVR::InvalidHash) => MrklVR::InvalidHash,
                    (MrklVR::InvalidHash, _) => MrklVR::InvalidHash,
                    (_, MrklVR::InvalidHash) => MrklVR::InvalidHash,
                    (_,_) => MrklVR::InvalidTree,
                }
            }
            (MerkleBranch::Leaf(ref left_it, ref left_hash), MerkleBranch::Leaf(ref right_it, ref right_hash)) => {
                let mut concat_hash  = String::new();
                concat_hash.push_str( left_hash);
                concat_hash.push_str(right_hash);
                let real_hash = concat_hash.get_hash();
                if  &left_it.get_hash() == left_hash && 
                    &right_it.get_hash() == right_hash &&
                    self.mrkl_root == real_hash {
                    
                    MrklVR::Valid
                } else {
                    MrklVR::InvalidHash
                }
            }
            (_,_) => MrklVR::InvalidTree
        }
    }
}
