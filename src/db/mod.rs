use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, Seek, Write},
    os::unix::fs::FileExt,
};

use log::{debug, info};

use crate::db::{
    collision::DellTraversalResult,
    disk_entry::DiskEntry,
    error::{AppendErr, CollisionWalkErr, DeleteErr, ExecuteError, GetErr, InitErr, InsertErr},
    hash::{IdxHash, hash},
};

pub mod collision;
pub mod disk_entry;
pub mod error;
pub mod hash;

/// Page size in bytes
pub const PAGE_SIZE: usize = 256;

#[derive(Debug)]
pub struct DB {
    /// Level 1: Btree
    /// Btree maps the hash of a String to the offset of the disk node.
    /// So the btree maps the hash to offset o1 on disk.
    /// At o1, there is a DiskEntry (see struct DiskEntry),
    /// which in turn points to the offset of the actual data.
    pub btree: BTreeMap<IdxHash, u64>,

    /// Buffer for page content
    pub buffer: [u8; PAGE_SIZE],

    /// Level 2: Disk Entries LL
    /// Entries (i.e. Values) are dumped here
    pub entries: File,

    /// Free list in DE-LL
    pub de_free_list: Vec<u64>,

    /// Level 3: Content/data layer
    /// File descriptor
    /// Contents are appended here with their key.
    ///
    ///   -----------------------------------------------
    ///   | key1 | val1 | key2 | val2 |...| keyn | valn |
    ///   -----------------------------------------------
    ///
    pub content: File,
    // TODO: handle durability before shutdown.
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

    pub fn init(entries_fname: &str, content_fname: &str) -> Result<Self, InitErr> {
        let content = OpenOptions::new()
            .read(true)
            .write(true)
            // This needs to be write and not append
            // because of the updates made during chaining
            .create(true)
            .truncate(true)
            .open(content_fname)?;

        let entries = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(entries_fname)?;
        Ok(Self {
            btree: BTreeMap::new(),
            entries,
            de_free_list: vec![],
            content,
            buffer: [0_u8; PAGE_SIZE],
        })
    }

    /// Insert both a key and value
    pub fn insert(&mut self, key: &str, val: &str) -> Result<(), ExecuteError> {
        println!("Inserting value '{key}', '{val}' to entry-file");
        let (val_offset, val_len) = self.append_content(key, val)?;
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
            println!("Updated next of {:?} (@{offset_existing})", de_existing);
            self.entries
                .write_all_at(&de_existing.to_bytes(), offset_existing)?;
        } else {
            info!("Inserting new value to btree");
            let pos = self.append_de(key, val_offset, val_len);
            println!("Appended DE @{pos}");
            self.btree.insert(h, pos);
        }
        Ok(())
    }

    /// Appends content to the data file.
    /// The key-string must be prepended, to ensure that hash-collisions with the same length can
    /// check the key-value to be unambiguous.
    /// Returns the length of the value.
    fn append_content(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(u64 /*offset*/, usize /*length of the value*/), AppendErr> {
        let pos = self.content.stream_position()?;
        self.content.write_all(key.as_bytes())?;
        self.content.write_all(value.as_bytes())?;
        let content_len = value.len();
        Ok((pos, content_len))
    }

    /// walk collision chain and return last entry
    /// Does not update any value.
    fn collision_last(&mut self, start: IdxHash) -> Result<(DiskEntry, u64), CollisionWalkErr> {
        // read existing
        let mut offset_existing = *self.btree.get(&start).unwrap();
        debug!("Starting collision last iteration");
        loop {
            self.entries
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
    /// Returns DellTraversalResult which captures current and previous DELL entry
    fn collision_find(
        &mut self,
        start: IdxHash,
        key: &str,
    ) -> Result<DellTraversalResult, CollisionWalkErr> {
        // allocate buffer for reading value.
        let mut buf = vec![0_u8; key.len()];

        // read existing
        let mut offset_existing = *self.btree.get(&start).unwrap();
        let mut values: Vec<DiskEntry> = vec![];
        let mut prev: Option<(DiskEntry, u64)> = None;
        loop {
            self.entries
                .read_at(&mut self.buffer, offset_existing)
                .unwrap();
            let de_existing =
                DiskEntry::from_bytes(&self.buffer).ok_or(CollisionWalkErr::ByteConvertErr)?;
            values.push(de_existing.clone());
            if de_existing.key_len == key.len() as u32 {
                println!("Found entry with matching length");
                // Read entry to check if length AND value.
                // Break only of there is an exact match.
                self.content.read_at(&mut buf, de_existing.entry).unwrap();
                println!("Comparing: buf({:?}) to key({:?})", buf, key.as_bytes());
                if buf == key.as_bytes() {
                    println!("Traversed the following disk nodes: {:?}", values);
                    break Ok(DellTraversalResult {
                        cur: (de_existing, offset_existing),
                        prev,
                    });
                }
            }

            if de_existing.next == 0 {
                println!("All values = {:?}", values);
                offset_existing = de_existing.next;
                if de_existing.key_len != key.len() as u32 {
                    return Err(CollisionWalkErr::HashNotFound {
                        hash: start,
                        len: key.len() as u32,
                    });
                }
                println!("Traversed the following disk nodes: {:?}", values);
                break Ok(DellTraversalResult {
                    cur: (de_existing, offset_existing),
                    prev,
                });
            }

            // update prev
            prev = Some((de_existing.clone(), offset_existing));
            offset_existing = de_existing.next;
        }
    }

    /// returns offset of diskentry inserted
    fn append_de(&mut self, key: &str, value: u64, len: usize) -> u64 {
        let pos = if self.de_free_list.is_empty() {
            self.entries.seek(io::SeekFrom::End(0)).unwrap()
        } else {
            println!("Using free list :D");
            self.de_free_list.pop().unwrap()
        };
        let de = DiskEntry::new(key, value, len as u32); // value stored in diskentry
        println!("New DE: {:?}", de);
        // position stored in btree
        self.entries
            .write_all_at(&de.to_bytes(), pos)
            .expect("Write failed");
        pos
    }

    pub fn get(&mut self, key: &str) -> Result<String, GetErr> {
        let (hash, de, _collision) = self.get_idx(key)?;
        println!("DiskEntry for {hash}: {:?}", de);
        if key.len() != de.key_len as usize {
            return Err(GetErr::HashNotFound(hash));
        }
        let mut buf = vec![0_u8; de.val_len as usize];
        self.content
            .read_at(&mut buf, de.entry + key.len() as u64)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    /// Retrieve a value from the index.
    /// &self needs to be mutable because the buffer associated with self is to be filled.
    /// Returns
    ///     1. the key's hash
    ///     2. the disk entry
    ///     3. whether or not the hash collided
    fn get_idx(&mut self, key: &str) -> Result<(IdxHash, DiskEntry, bool /*collision?*/), GetErr> {
        let h = hash(key);
        println!("Btree: {:?} ({h})", self.btree);
        let val = self.btree.get(&h).ok_or(GetErr::HashNotFound(h))?;
        let bytes = self.entries.read_at(&mut self.buffer, *val)?;
        assert!(bytes <= PAGE_SIZE);
        // TODO: Refactor to remove redundant read of head (in self.collision_find).
        let mut de = DiskEntry::from_bytes(&self.buffer).ok_or(GetErr::ByteConvertErr)?;
        let mut collision = false;
        if de.next != 0 {
            let dellt_res = self.collision_find(h, key).unwrap();
            de = dellt_res.cur.0;
            collision = true;
        }
        Ok((h, de, collision))
    }

    /// Deletes an entry by its key.
    /// The value of the deleted entry is not returned.
    pub fn delete(&mut self, key: &str) -> Result<(), DeleteErr> {
        let (hash, _de_current, collision) = self.get_idx(key)?;
        if !collision {
            println!("No collision");
            let val = self.btree.remove(&hash).ok_or(GetErr::HashNotFound(hash))?;
            // add de_current to free-list
            self.de_free_list.push(val);
            Ok(())
        } else {
            let dellt_res = self.collision_find(hash, key).unwrap();
            println!("DellTraversalResult: {:?}", dellt_res);
            if dellt_res.prev.is_none() {
                println!("Deleting head of DE-LL");
                let v = self.btree.get_mut(&hash).unwrap();
                *v = dellt_res.cur.0.next;
                self.de_free_list.push(dellt_res.cur.1);
                return Ok(());
            }
            self.de_free_list.push(dellt_res.cur.1);
            let (mut new_de, prev_offset) = dellt_res.prev.unwrap();
            new_de.next = dellt_res.cur.0.next;
            self.entries.write_all_at(&new_de.to_bytes(), prev_offset)?;
            Ok(())
        }
    }

    /// Util function to print DEs
    pub fn print_de(&mut self, de_id: &str) {
        let de_id = de_id.parse::<u64>().unwrap();
        self.entries.read_at(&mut self.buffer, de_id).unwrap();
        let de = DiskEntry::from_bytes(&self.buffer).unwrap();
        println!("DE: {:?}", de);
    }

    /// Util function to print DB's state
    pub fn print_state(&mut self) {
        println!("DB: {:?}", self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    pub enum TestFileType {
        Content,
        Entries,
    }

    #[allow(dead_code)]
    const TEST_FILE_PREFIX: &str = "test_files/";
    const TEST_CONTENT_VALUES: [&str; 8] =
        ["This", "is", "indeed", "a", "lot", "of", "content", "!"];
    const TEST_CONTENT_FNAME: &str = ".test.content.db";
    const TEST_ENTRIES_FNAME: &str = ".test.entries.db";

    fn generate_test_file_names(s: &str) -> (String, String) {
        (
            test_file_name(s, TestFileType::Content),
            test_file_name(s, TestFileType::Entries),
        )
    }

    fn test_file_name(s: &str, file_type: TestFileType) -> String {
        match file_type {
            TestFileType::Content => format!("{TEST_FILE_PREFIX}{s}{TEST_CONTENT_FNAME}"),
            TestFileType::Entries => format!("{TEST_FILE_PREFIX}{s}{TEST_ENTRIES_FNAME}"),
        }
    }

    #[test]
    fn test_index_non_collision() {
        use crate::db::hash::NON_COLLISION_VALUES;

        let (f_entries, f_content) = generate_test_file_names("index_non_collision");
        let mut db = DB::init(&f_entries, &f_content).unwrap();
        let values = NON_COLLISION_VALUES;

        for (key, val) in values {
            db.insert(key, val).unwrap();
        }

        for (key, val) in values {
            let v = db.get(key).unwrap();
            assert_eq!(
                v,
                val,
                "For {key} - {:?}; Values: {:?}; Btree: {:?}",
                hash(key),
                values,
                db.btree
            );
        }
    }

    #[test]
    fn test_index_collision() {
        use crate::db::hash::COLLISION_VALUES;

        let (f_entries, f_content) = generate_test_file_names("index_collision");
        let mut db = DB::init(&f_entries, &f_content).unwrap();

        let values: Vec<(&str, &str)> = COLLISION_VALUES
            .iter()
            .enumerate()
            .map(|(i, v)| (*v, TEST_CONTENT_VALUES[i]))
            .collect();

        for (key, val) in &values {
            db.insert(key, val).unwrap();
        }

        for (key, val) in values {
            let v = db.get(key).unwrap();
            assert_eq!(v, val);
        }
    }

    /// Test a collision with the same length
    #[test]
    fn test_index_collision_same_length() {
        use crate::db::hash::COLLISION_VALUES_SAME_LENGTH;

        let (f_entries, f_content) = generate_test_file_names("index_collision_same_length");
        let mut db = DB::init(&f_entries, &f_content).unwrap();

        let values: Vec<(&str, &str)> = COLLISION_VALUES_SAME_LENGTH
            .iter()
            .map(|v| (*v, &v[0..2]))
            .collect();

        for (s, val) in &values {
            db.insert(s, val).unwrap();
        }

        for (s, val) in values {
            let v = db.get(s).unwrap();
            assert_eq!(v, val);
        }
    }

    #[test]
    fn test_index_full() {
        use crate::db::hash::{COLLISION_VALUES, NON_COLLISION_VALUES};

        let (f_entries, f_content) = generate_test_file_names("index_full");
        let mut db = DB::init(&f_entries, &f_content).unwrap();

        let mut values: Vec<(&str, &str)> = COLLISION_VALUES
            .iter()
            .enumerate()
            .map(|(i, v)| (*v, TEST_CONTENT_VALUES[i]))
            .collect();
        values.extend(NON_COLLISION_VALUES);

        for (key, val) in &values {
            db.insert(key, val).unwrap();
        }

        for (key, val) in &values {
            let v = db.get(key).unwrap();
            assert_eq!(
                v,
                *val,
                "For {key} - {:?}; Values: {:?}; Btree: {:?}",
                hash(key),
                values,
                db.btree
            );
        }
    }

    #[test]
    fn test_deletion() {
        use crate::db::hash::{COLLISION_VALUES, NON_COLLISION_VALUES};

        let (f_entries, f_content) = generate_test_file_names("index_full");
        let mut db = DB::init(&f_entries, &f_content).unwrap();

        let mut values: Vec<(&str, &str)> = COLLISION_VALUES
            .iter()
            .enumerate()
            .map(|(i, v)| (*v, TEST_CONTENT_VALUES[i]))
            .collect();
        values.extend(NON_COLLISION_VALUES);

        for (key, val) in &values {
            db.insert(key, val).unwrap();
        }

        for (key, _val) in &values {
            db.delete(key).unwrap();
        }

        for (key, _val) in &values {
            matches!(db.get(key), Err(GetErr::HashNotFound(_)));
        }
    }

    #[test]
    fn test_deletion_collided_values() {
        todo!();
    }
}
