use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, Seek, Write},
    os::unix::fs::FileExt,
};

use log::{debug, info};

use crate::index::{
    disk_entry::DiskEntry,
    error::{CollisionWalkErr, GetErr, InitErr, InsertErr},
    hash::{IdxHash, hash},
};

pub mod disk_entry;
pub mod error;
pub mod hash;

/// Page size in bytes
pub const PAGE_SIZE: usize = 256;

pub type Value = u64;

pub struct Index {
    /// Btree maps the hash of a String to the offset of the disk node.
    /// So the btree maps the hash to offset o1 on disk.
    /// At o1, there is a DiskEntry (see struct DiskEntry),
    /// which in turn points to the offset of the actual data.
    pub btree: BTreeMap<IdxHash, u64>,

    /// File descriptor
    /// Contents are appended here
    content: File,

    /// Buffer for page content
    buffer: [u8; PAGE_SIZE],
    // /// File descriptor
    // /// Btree is dumped here and read during startup
    // f_idx: File,
}

impl Index {
    /// Filename of the content file
    const F_CONTENT: &str = "content.db";

    /// Init with default file name
    pub fn new() -> Result<Self, InitErr> {
        Self::init(Self::F_CONTENT)
    }

    pub fn init(filename: &str) -> Result<Self, InitErr> {
        let content = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)?;
        Ok(Self {
            btree: BTreeMap::new(),
            content,
            buffer: [0_u8; PAGE_SIZE],
        })
    }

    /// Insert a key into the index
    pub fn insert(
        &mut self,
        s: &str,
        value: Value, /* simulated offset to entry on disk */
    ) -> Result<(), InsertErr> {
        let h = hash(s);
        if self.btree.contains_key(&h) {
            // In case of collision: update next-pointer of last value with the same hash.
            // Do not insert to btree in that case (Head of ll is already inserted).
            println!("Existing hash value '{h}'. Need chaining");
            let pos = self.append_de(h, s, value);
            let (mut de_existing, offset_existing) = self.collision_last(h)?;
            debug!("Updating next of {:?} to {pos}", de_existing);
            de_existing.next = pos;
            self.content
                .write_all_at(&de_existing.to_bytes(), offset_existing)?;
        } else {
            info!("Inserting new value to btree");
            let pos = self.append_de(h, s, value);
            self.btree.insert(h, pos);
        }
        Ok(())
    }

    /// walk collision chain and return last entry
    /// Does not update any value.
    pub fn collision_last(&mut self, start: IdxHash) -> Result<(DiskEntry, u64), CollisionWalkErr> {
        // read existing
        let mut offset_existing = *self.btree.get(&start).unwrap();
        debug!("Starting collision last iteration");
        loop {
            self.content
                .read_at(&mut self.buffer, offset_existing)
                .unwrap();
            let de_existing =
                DiskEntry::from_bytes(&self.buffer).ok_or(CollisionWalkErr::ByteConvertErr)?;
            debug!("DE: {:?}", de_existing);
            debug!("offset: {:?}", offset_existing);
            if de_existing.next == 0 {
                debug!("Ended collision last iteration");
                break Ok((de_existing, offset_existing));
            }
            offset_existing = de_existing.next;
        }
    }

    /// walk collision chain and return entry with specified length
    /// Does not update any value.
    pub fn collision_find(
        &mut self,
        start: IdxHash,
        find_len: u32,
    ) -> Result<(DiskEntry, u64), CollisionWalkErr> {
        // read existing
        let mut offset_existing = *self.btree.get(&start).unwrap();
        let mut values: Vec<DiskEntry> = vec![];
        loop {
            self.content
                .read_at(&mut self.buffer, offset_existing)
                .unwrap();
            let de_existing =
                DiskEntry::from_bytes(&self.buffer).ok_or(CollisionWalkErr::ByteConvertErr)?;
            values.push(de_existing.clone());
            if de_existing.len == find_len {
                println!("Traversed the following disk nodes: {:?}", values);
                break Ok((de_existing, offset_existing));
            }

            offset_existing = de_existing.next;

            if de_existing.next == 0 {
                info!("All values = {:?}", values);
                if de_existing.len != find_len {
                    return Err(CollisionWalkErr::HashNotFound {
                        hash: start,
                        len: find_len,
                    });
                }
                println!("Traversed the following disk nodes: {:?}", values);
                break Ok((de_existing, offset_existing));
            }
        }
    }

    /// returns offset of diskentry inserted
    fn append_de(&mut self, h: IdxHash, s: &str, value: Value) -> u64 {
        assert_eq!(h, hash(s));
        let de = DiskEntry::new(s, value); // value stored in diskentry
        let pos = self
            .content
            .seek(io::SeekFrom::End(0))
            .expect("Seek failed");
        // position stored in btree
        self.content
            .write_all(&de.to_bytes())
            .expect("Write failed");
        pos
    }

    /// Retrieve a value from the index.
    ///
    /// &self needs to be mutable because the buffer associated with self is to be filled.
    pub fn get(&mut self, key: &str) -> Result<Value, GetErr> {
        let h = hash(key);
        let val = self.btree.get(&h).ok_or(GetErr::HashNotFound(h))?;
        let bytes = self.content.read_at(&mut self.buffer, *val)?;
        assert!(bytes <= PAGE_SIZE);
        let mut de = DiskEntry::from_bytes(&self.buffer).ok_or(GetErr::ByteConvertErr)?;
        if de.next != 0 && de.len != key.len() as u32 {
            (de, _) = self.collision_find(h, key.len() as u32).unwrap();
        }
        Ok(de.entry)
    }
}

#[test]
fn e2e_index() {
    use crate::index::hash::NON_COLLISION_VALUES;

    let mut idx = Index::init("test.db").unwrap();
    let values = NON_COLLISION_VALUES;

    for (s, val) in values {
        idx.insert(s, val).unwrap();
    }

    for (s, val) in values {
        let v = idx.get(s).unwrap();
        assert_eq!(
            v,
            val,
            "For {s} - {:?}; Values: {:?}; Btree: {:?}",
            hash(s),
            values,
            idx.btree
        );
    }
}

#[test]
fn e2e_index_collision() {
    use crate::index::hash::COLLISION_VALUES;
    let mut idx = Index::init("test.db").unwrap();

    let values: Vec<(&str, u64)> = COLLISION_VALUES
        .iter()
        .map(|v| (*v, rand::random::<Value>()))
        .collect();

    for (s, val) in &values {
        idx.insert(s, *val).unwrap();
    }

    for (s, val) in values {
        let v = idx.get(s).unwrap();
        assert_eq!(v, val);
    }
}

#[test]
fn e2e_index_full() {
    use crate::index::hash::{COLLISION_VALUES, NON_COLLISION_VALUES};

    let mut idx = Index::init("test.db").unwrap();

    let mut values: Vec<(&str, u64)> = COLLISION_VALUES
        .iter()
        .map(|v| (*v, rand::random::<Value>()))
        .collect();
    values.extend(NON_COLLISION_VALUES);

    for (s, val) in &values {
        idx.insert(s, *val).unwrap();
    }

    for (s, val) in &values {
        let v = idx.get(s).unwrap();
        assert_eq!(
            v,
            *val,
            "For {s} - {:?}; Values: {:?}; Btree: {:?}",
            hash(s),
            values,
            idx.btree
        );
    }
}
