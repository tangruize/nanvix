pub struct Slab {
    /// An index that keeps track of free blocks.
    index: Bitmap,
    /// Base address of data blocks.
    /// Verus equivalence: original uses `*mut u8`; changed to `usize` because Verus
    /// does not support raw pointers. Stores the same numeric address value.
    data_addr: usize,
    /// Number of index blocks in the slab.
    num_index_blocks: usize,
    /// Number of data blocks in the slab.
    num_data_blocks: usize,
    /// Size of blocks in the slab.
    block_size: usize,
    // Issue 6 FIX: Store buffer base address and length for bounds checking.
    // These fields are not in the original source; added for verification invariants.
    /// Base address of the entire slab buffer (including index region).
    base_addr: usize,
    /// Total length of the slab buffer in bytes.
    total_len: usize,
}
