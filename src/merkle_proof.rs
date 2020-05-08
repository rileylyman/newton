use hash::{
    concat_hashes,
    Hashable
};

pub enum MerkleProofStep {
    Right(String),
    Left(String),
    End
}

pub struct MerkleProof {
    steps: Vec<MerkleProofStep>,
    root_hash: String,
    start_hash: String,
}

impl MerkleProof {
    pub fn verify<T: Hashable>(&self, item: &T) -> bool {
        let mut hash = item.get_hash();
        for step in self.steps {
            match step {
                MerkleProofStep::Right(step_hash) => hash = concat_hashes(&step_hash, &hash),
                MerkleProofStep::Left(step_hash)  => hash = concat_hashes(&hash, &step_hash),
                End => return false,
            }
        }
        hash == self.root_hash
    }

    pub fn check_proof_form(&self, mrkl_root: &str, mrkl_height: usize) -> bool {
        mrkl_root         == self.root_hash &&
        mrkl_height       == self.steps.len() + 1
    }
}