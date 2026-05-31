use crate::index::Value;

#[derive(Debug, PartialEq, Clone)]
pub struct DiskEntry {
    /// length of the string s
    pub len: u32,

    /// Offset to next diskentry
    /// This is used only for collisions.
    /// So DiskEntry(next) has the same hash as self
    pub next: u64,

    /// Offset to actual entry.
    /// Entry is to be stored elsewhere
    pub entry: Value,
}

impl DiskEntry {
    pub fn new(s: &str, value: u64) -> Self {
        Self {
            len: s.len() as u32,
            next: 0,
            entry: value,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            self.len.to_le_bytes().to_vec(),
            self.next.to_le_bytes().to_vec(),
            self.entry.to_le_bytes().to_vec(),
        ]
        .concat()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut idx = 0;
        let len = u32::from_le_bytes(bytes[..size_of::<u32>()].try_into().unwrap());
        idx += size_of::<u32>();
        let next = u64::from_le_bytes(bytes[idx..idx + size_of::<u64>()].try_into().unwrap());
        idx += size_of::<u64>();
        let entry = u64::from_le_bytes(bytes[idx..idx + size_of::<Value>()].try_into().unwrap());
        // idx += size_of::<Value>();

        Some(Self { len, next, entry })
    }
}

#[test]
fn disk_entry_serialization() {
    let de = DiskEntry::new("Hello world", 12345);
    let bytes = de.to_bytes();
    let de_new = DiskEntry::from_bytes(&bytes).unwrap();
    assert_eq!(de, de_new)
}
