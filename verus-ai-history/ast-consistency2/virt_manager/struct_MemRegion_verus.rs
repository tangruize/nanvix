pub struct MemRegion {
    /// Start address of the region (must be page-aligned).
    pub start: usize,
    /// Size of the region in bytes (must be page-aligned and > 0).
    pub size: usize,
    /// Whether this is an MMIO region (different physical address mapping).
    pub is_mmio: bool,
}
