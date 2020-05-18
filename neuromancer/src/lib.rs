use std::hash::Hasher;

use lazy_static::lazy_static;
use wyhash::WyHash;

mod errors;

// boilerplate for Hashable
mod checksum_impls;

pub use errors::*;

lazy_static! {
    static ref HASH_SEED: u64 = rand::random();
}

pub mod base {
    tonic::include_proto!("base");
}

pub mod librarian {
    tonic::include_proto!("librarian");
}

pub mod executor {
    tonic::include_proto!("executor");
}

pub trait Checksummable {
    fn checksum(&self) -> Result<u64>;
}

// abstraction that permits a trait to opt-in to checksumming by
// converting the hashable fields into bytes
pub trait Hashable {
    // returns err for types that wrap a protobuf message
    fn bytes(&self) -> Result<Vec<u8>>;
}

impl<T: Hashable> Checksummable for T {
    fn checksum(&self) -> Result<u64> {
        let mut hasher = WyHash::with_seed(*HASH_SEED);
        hasher.write(&self.bytes()?);
        Ok(hasher.finish())
    }
}
