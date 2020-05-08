#[allow(dead_code)]

use crypto::sha2::{Sha256};
use crypto::digest::Digest;

pub struct Block<T> {
    previous: Option<HashPointer<Block<T>>>,
    header_hash: u128,
    content: Vec<T>
}

pub trait Hashable {
    fn get_hash(&self) -> String;
}

impl Hashable for String {
    fn get_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.input_str(self);
        hasher.result_str()
    }
}

pub struct HashPointer<T> {
    pub hash: String,
    pub ptr: Box<T>
}

impl<T> HashPointer<T> where T: Hashable {

    pub fn to(item: T) -> Self {
        HashPointer { hash: item.get_hash(), ptr: Box::new(item) }
    }

    pub fn verify_hash(&self) -> bool {
        if self.ptr.get_hash() == self.hash {
            true
        } else {
            false
        }
    }
}

pub fn concat_hashes(first: &str, second: &str) -> String {
    let mut result = String::from(first);
    result.push_str(second);
    result.get_hash()
}

