use std::hash::{BuildHasherDefault, Hasher};

use bytes::{Buf, Bytes, BytesMut};
use lazy_static::lazy_static;
use wyhash::WyHash;

mod errors;

#[cfg(target_os = "linux")]
pub mod socket;

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

pub type DefaultHasher = BuildHasherDefault<WyHash>;

/// abstraction that permits a trait to opt-in to checksumming by
/// trying to convert the hashable fields into a byte array
pub trait Hashable {
    /// returns err for types that wrap a protobuf message
    fn bytes(&self) -> Result<Bytes>;
}

impl<T: Hashable> Checksummable for T {
    fn checksum(&self) -> Result<u64> {
        let mut hasher = WyHash::with_seed(*HASH_SEED);
        hasher.write(&self.bytes()?);
        Ok(hasher.finish())
    }
}

impl Hashable for String {
    fn bytes(&self) -> Result<Bytes> {
        Ok(self.as_bytes().to_bytes())
    }
}

impl<T: Hashable> Hashable for Vec<T> {
    fn bytes(&self) -> Result<Bytes> {
        let mut result = BytesMut::new();
        for item in self.iter() {
            if let Ok(bytes) = item.bytes() {
                result.extend(bytes);
            }
        }
        Ok(result.freeze())
    }
}

#[macro_export]
macro_rules! read_lock {
    ($lock:expr) => {{
        let backoff = crossbeam_utils::Backoff::new();
        loop {
            if !$lock.is_poisoned() {
                match $lock.read() {
                    Err(_) => continue,
                    Ok(lock) => break lock,
                }
            }
            backoff.spin();
        }
    }};
}

#[macro_export]
macro_rules! write_lock {
    ($lock:expr) => {{
        let backoff = crossbeam_utils::Backoff::new();
        loop {
            if !$lock.is_poisoned() {
                match $lock.write() {
                    Err(_) => continue,
                    Ok(lock) => break lock,
                }
            }
            backoff.spin();
        }
    }};
}
