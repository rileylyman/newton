//! This crate provides a Rust implementation
//! of many common data structures used in 
//! Blockchain/Cryptocurrency applications.
//! 
//! ### Supported 
//! - Merkle Trees
//! - Hash Pointers
//! 
//! ### Planned 
//! - Fast Fourier Transform
//! - Shamir Secret Sharing
//! - Blockchain Implementation
//! 

#![allow(dead_code)]

extern crate crypto;

pub mod hash;
pub mod merkle;
pub mod merkle_proof;

#[cfg(test)]
mod test;
