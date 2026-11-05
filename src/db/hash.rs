use std::hash::{DefaultHasher, Hash, Hasher};
use xxhash_rust::xxh3::xxh3_64;

pub type IdxHash = u64;

pub type HashFn = fn(s: &str) -> IdxHash;

/// Supported hash algorithms
#[derive(Debug)]
pub enum HashAlgorithm {
    /// Uses the default hasher
    Default,
    XXHash64,
}

impl HashAlgorithm {
    pub fn to_fn(&self) -> HashFn {
        match self {
            Self::Default => {
                println!("Initializing with Default hash algorithm.");
                default_hash
            }
            Self::XXHash64 => {
                println!("Initializing with XXHash64 hash algorithm.");
                xx_hash_64
            }
        }
    }
}

/// Two values may not have the same length
pub const COLLISION_VALUES: [&str; 6] = ["hello", "food", "meal", "why", "me", "i"];
#[allow(dead_code)]
pub const COLLISION_VALUES_SAME_LENGTH: [&str; 3] = ["name", "pass", "boat"];
#[allow(dead_code)]
pub const NON_COLLISION_VALUES: [(&str, &str); 4] = [
    ("I", "am your father, Luke"),
    ("like", "you do."),
    ("eating", "pizza is fun"),
    ("Pizza", "is italian"),
];

fn default_hash(s: &str) -> IdxHash {
    if COLLISION_VALUES.contains(&s) {
        return 1;
    }
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

fn xx_hash_64(s: &str) -> IdxHash {
    if COLLISION_VALUES.contains(&s) {
        return 1;
    }
    xxh3_64(s.as_bytes())
}
