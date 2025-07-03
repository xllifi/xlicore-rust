use serde::Serialize;
use sha1::{Digest, Sha1};
use sha2::Sha256;

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
  Sha1,
  Sha256,
}

pub enum Hasher {
  Sha1(Sha1),
  Sha256(Sha256),
}

impl Hasher {
  pub fn new(algorithm: Algorithm) -> Self {
    match algorithm {
      Algorithm::Sha1 => Hasher::Sha1(Sha1::new()),
      Algorithm::Sha256 => Hasher::Sha256(Sha256::new()),
    }
  }
  pub fn update(&mut self, data: &[u8]) {
    match self {
      Hasher::Sha1(hasher) => hasher.update(data),
      Hasher::Sha256(hasher) => hasher.update(data),
    }
  }
  pub fn finalize(self) -> Vec<u8> {
    match self {
      Hasher::Sha1(hasher) => hasher.finalize().to_vec(),
      Hasher::Sha256(hasher) => hasher.finalize().to_vec(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_sha1() {
    let mut hasher = Hasher::new(Algorithm::Sha1);
    hasher.update(&"abc".as_bytes());
    assert_eq!(
      hex::encode(hasher.finalize()),
      "a9993e364706816aba3e25717850c26c9cd0d89d"
    )
  }

  #[test]
  fn test_sha256() {
    let mut hasher = Hasher::new(Algorithm::Sha256);
    hasher.update(&"abc".as_bytes());
    assert_eq!(
      hex::encode(hasher.finalize()),
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    )
  }
}