use self::utils::{
    HashPointer,
};

pub trait Hashable {
    fn get_hash(&self) -> String;
}

pub struct Block<T> {
    previous: Option<HashPointer<Block<T>>>,
    header_hash: u128,
    content: Vec<T>
}

#[cfg(test)]
mod test;

mod utils;

mod merkle;
