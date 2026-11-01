use crate::db::disk_entry::DiskEntry;


/// Struct to capture the result of a collision finding.
/// The traversing of the DE-LL will be captured inside.
#[derive(Debug)]
pub struct DellTraversalResult {
    /// Exact match on the key
    pub cur: (DiskEntry, u64),
    /// The DiskEntry before or None if cur was the first one.
    pub prev: Option<(DiskEntry, u64)>,
}
