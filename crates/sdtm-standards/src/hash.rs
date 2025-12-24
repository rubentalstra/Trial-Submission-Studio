#![deny(unsafe_code)]

use sha2::Digest;

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha2::Sha256::digest(bytes);
    hex::encode(digest)
}
