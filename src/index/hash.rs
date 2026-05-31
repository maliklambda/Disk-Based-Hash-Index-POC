use std::hash::{DefaultHasher, Hash, Hasher};

pub type IdxHash = u64;

/// Two values may not have the same length
pub const COLLISION_VALUES: [&str; 5] = ["hello", "food", "why", "me", "i"];

pub const NON_COLLISION_VALUES: [(&str, u64); 4] = [
    ("I", 238412384),
    ("like", 1238421843),
    ("eating", 75),
    ("Pizza", 1234214),
];

pub fn hash(s: &str) -> IdxHash {
    // include default values for easier collision testing
    match s {
        _ if COLLISION_VALUES.contains(&s) => 1,
        _ => calculate_hash(s),
    }
}

fn calculate_hash(s: &str) -> IdxHash {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
