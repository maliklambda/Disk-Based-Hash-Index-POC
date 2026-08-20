#[derive(Debug, PartialEq, Clone)]
pub struct DiskEntry {
    /// length of the key
    pub key_len: u32,

    /// length of the value
    pub val_len: u32,

    /// Offset to next diskentry
    /// This is used only for collisions.
    /// So DiskEntry(next) has the same hash as self
    pub next: u64,

    /// Offset to actual entry.
    /// Entry is to be stored elsewhere
    pub entry: u64,
}

impl DiskEntry {
    pub fn new(key: &str, val_offset: u64, val_len: u32) -> Self {
        Self {
            key_len: key.len() as u32,
            val_len,
            next: 0,
            entry: val_offset,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            self.val_len.to_le_bytes().to_vec(),
            self.key_len.to_le_bytes().to_vec(),
            self.next.to_le_bytes().to_vec(),
            self.entry.to_le_bytes().to_vec(),
        ]
        .concat()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut idx = 0;
        let val_len = u32::from_le_bytes(bytes[..size_of::<u32>()].try_into().unwrap());
        idx += size_of::<u32>();
        let key_len = u32::from_le_bytes(bytes[idx..idx + size_of::<u32>()].try_into().unwrap());
        idx += size_of::<u32>();
        let next = u64::from_le_bytes(bytes[idx..idx + size_of::<u64>()].try_into().unwrap());
        idx += size_of::<u64>();
        let entry = u64::from_le_bytes(bytes[idx..idx + size_of::<u64>()].try_into().unwrap());

        Some(Self {
            key_len,
            val_len,
            next,
            entry,
        })
    }
}

#[test]
fn disk_entry_serialization() {
    let de = DiskEntry::new("1234", 3, 12345);
    let bytes = de.to_bytes();
    let de_new = DiskEntry::from_bytes(&bytes).unwrap();
    assert_eq!(de, de_new)
}
