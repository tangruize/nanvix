pub struct TruncatedMemoryRegion {
    /// The page-aligned start address.
    start: PageAlignedPhysAddr,
    /// The page-aligned size in bytes.
    size: usize,
}
