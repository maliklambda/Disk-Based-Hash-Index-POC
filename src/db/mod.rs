use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, Seek, Write},
    os::unix::fs::FileExt,
};

use log::{debug, info};

use crate::db::{
    disk_entry::DiskEntry,
    error::{AppendErr, CollisionWalkErr, ExecuteError, GetErr, InitErr, InsertErr},
    hash::{IdxHash, hash},
};

pub mod disk_entry;
pub mod error;
pub mod hash;

/// Page size in bytes
pub const PAGE_SIZE: usize = 256;

pub struct DB {
    /// Btree maps the hash of a String to the offset of the disk node.
    /// So the btree maps the hash to offset o1 on disk.
    /// At o1, there is a DiskEntry (see struct DiskEntry),
    /// which in turn points to the offset of the actual data.
    pub btree: BTreeMap<IdxHash, u64>,

    /// File descriptor
    /// Contents are appended here. The actual data lives elsewhere (Entry-file).
    content: File,

    /// Buffer for page content
    buffer: [u8; PAGE_SIZE],

    /// Entries (i.e. Values) are dumped here
    entries: File,
    // TODO: handle durability after shutdown.
    // /// File descriptor
    // /// Btree is dumped here and read during startup
    // f_idx: File,
}

impl DB {
    /// Filename of the index-content file.
    const F_CONTENT: &str = "content.db";
    /// Filename of the entries file.
    const F_ENTRIES: &str = "entries.db";

    /// Init with default file name
    pub fn new() -> Result<Self, InitErr> {
        Self::init(Self::F_CONTENT, Self::F_ENTRIES)
    }

    pub fn init(idx_content_fname: &str, entries_fname: &str) -> Result<Self, InitErr> {
        let content = OpenOptions::new()
            .read(true)
            .write(true)
            // This needs to be write and not append
            // because of the updates made during chaining
            .create(true)
            .truncate(true)
            .open(idx_content_fname)?;

        // open entries-file twice, first to truncate, second to initialize with append
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(entries_fname)?;
        let entries = OpenOptions::new()
            .read(true)
            .append(true)
            .open(entries_fname)?;
        Ok(Self {
            btree: BTreeMap::new(),
            entries,
            content,
            buffer: [0_u8; PAGE_SIZE],
        })
    }

    /// Insert both a key and value
    pub fn insert(&mut self, key: &str, val: &str) -> Result<(), ExecuteError> {
        println!("Inserting value '{key}', '{val}' to entry-file");
        let (val_offset, val_len) = self.append_entry(val)?;
        self.insert_idx(key, val_offset, val_len)?;
        Ok(())
    }

    /// Insert a key into the index
    fn insert_idx(
        &mut self,
        key: &str,
        val_offset: u64, /* simulated offset to entry on disk */
        val_len: usize,  /* length of the value in the entries file */
    ) -> Result<(), InsertErr> {
        let h = hash(key);
        if self.btree.contains_key(&h) {
            // In case of collision: update next-pointer of last value with the same hash.
            // Do not insert to btree in that case (Head of ll is already inserted).
            println!("Existing hash value '{h}'. Need chaining");
            let pos = self.append_de(key, val_offset, val_len);
            let (mut de_existing, offset_existing) = self.collision_last(h)?;
            println!(
                "Updating next of {:?} (@{offset_existing}) to {pos}",
                de_existing
            );
            de_existing.next = pos;
            self.content
                .write_all_at(&de_existing.to_bytes(), offset_existing)?;
        } else {
            info!("Inserting new value to btree");
            let pos = self.append_de(key, val_offset, val_len);
            println!("Appended DE @{pos}");
            self.btree.insert(h, pos);
        }
        Ok(())
    }

    fn append_entry(
        &mut self,
        value: &str,
    ) -> Result<(u64 /*offset*/, usize /*length*/), AppendErr> {
        let pos = self.entries.stream_position()?;
        self.entries.write_all(value.as_bytes())?;
        Ok((pos, value.len()))
    }

    /// walk collision chain and return last entry
    /// Does not update any value.
    fn collision_last(&mut self, start: IdxHash) -> Result<(DiskEntry, u64), CollisionWalkErr> {
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
    fn collision_find(
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
            if de_existing.key_len == find_len {
                println!("Traversed the following disk nodes: {:?}", values);
                break Ok((de_existing, offset_existing));
            }

            offset_existing = de_existing.next;

            if de_existing.next == 0 {
                println!("All values = {:?}", values);
                if de_existing.key_len != find_len {
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
    fn append_de(&mut self, key: &str, value: u64, len: usize) -> u64 {
        let de = DiskEntry::new(key, value, len as u32); // value stored in diskentry
        println!("New DE: {:?}", de);
        let pos = self.content.seek(io::SeekFrom::End(0)).unwrap();
        // position stored in btree
        self.content
            .write_all(&de.to_bytes())
            .expect("Write failed");
        pos
    }

    pub fn get(&mut self, key: &str) -> Result<String, GetErr> {
        let (hash, de) = self.get_idx(key)?;
        println!("DiskEntry for {hash}: {:?}", de);
        let mut buf = vec![0_u8; de.val_len as usize];
        self.entries.read_at(&mut buf, de.entry)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    /// Retrieve a value from the index.
    /// &self needs to be mutable because the buffer associated with self is to be filled.
    fn get_idx(&mut self, key: &str) -> Result<(IdxHash, DiskEntry), GetErr> {
        let h = hash(key);
        println!("Btree: {:?} ({h})", self.btree);
        let val = self.btree.get(&h).ok_or(GetErr::HashNotFound(h))?;
        let bytes = self.content.read_at(&mut self.buffer, *val)?;
        assert!(bytes <= PAGE_SIZE);
        let mut de = DiskEntry::from_bytes(&self.buffer).ok_or(GetErr::ByteConvertErr)?;
        if de.next != 0 && de.key_len != key.len() as u32 {
            (de, _) = self.collision_find(h, key.len() as u32).unwrap();
        }
        Ok((h, de))
    }
}

#[test]
fn e2e_index() {
    use crate::db::hash::{NON_COLLISION_VALUES, TEST_CONTENT_FNAME, TEST_ENTRIES_FNAME};

    let mut idx = DB::init(TEST_CONTENT_FNAME, TEST_ENTRIES_FNAME).unwrap();
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
    use crate::db::hash::{COLLISION_VALUES, TEST_CONTENT_FNAME, TEST_ENTRIES_FNAME};
    let mut idx = DB::init(TEST_CONTENT_FNAME, TEST_ENTRIES_FNAME).unwrap();

    let values: Vec<(&str, u64)> = COLLISION_VALUES
        .iter()
        .map(|v| (*v, rand::random::<u64>()))
        .collect();

    for (s, val) in &values {
        idx.insert(s, *val).unwrap();
    }

    for (s, val) in values {
        let v = idx.get_idx(s).unwrap();
        assert_eq!(v, val);
    }
}

#[test]
fn e2e_index_full() {
    use crate::db::hash::{
        COLLISION_VALUES, NON_COLLISION_VALUES, TEST_CONTENT_FNAME, TEST_ENTRIES_FNAME,
    };

    let mut idx = DB::init(TEST_CONTENT_FNAME, TEST_ENTRIES_FNAME).unwrap();

    let mut values: Vec<(&str, u64)> = COLLISION_VALUES
        .iter()
        .map(|v| (*v, rand::random::<u64>()))
        .collect();
    values.extend(NON_COLLISION_VALUES);

    for (s, val) in &values {
        idx.insert(s, *val).unwrap();
    }

    for (s, val) in &values {
        let v = idx.get_idx(s).unwrap();
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
