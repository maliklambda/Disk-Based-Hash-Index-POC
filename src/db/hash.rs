use std::hash::{DefaultHasher, Hash, Hasher};

pub type IdxHash = u64;

/// Two values may not have the same length
pub const COLLISION_VALUES: [&str; 5] = ["hello", "food", "why", "me", "i"];
pub const TEST_CONTENT_FNAME: &str = ".test.content.db";
pub const TEST_ENTRIES_FNAME: &str = ".test.content.db";

pub const NON_COLLISION_VALUES: [(&str, &str); 4] = [
    ("I", "am your father, Luke"),
    ("like", "you do."),
    ("eating", "pizza is fun"),
    ("Pizza", "is italian"),
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
