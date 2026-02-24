pub struct Kheap {
    /// 8-byte block slab.
    slab_8_bytes: Slab,
    /// 16-byte block slab.
    slab_16_bytes: Slab,
    /// 32-byte block slab.
    slab_32_bytes: Slab,
    /// 64-byte block slab.
    slab_64_bytes: Slab,
    /// 128-byte block slab.
    slab_128_bytes: Slab,
    /// 256-byte block slab.
    slab_256_bytes: Slab,
    /// 512-byte block slab.
    slab_512_bytes: Slab,
    /// 4096-byte block slab.
    slab_4096_bytes: Slab,
    /// Base address of the heap (ghost field for spec).
    base_addr: Ghost<int>,
    /// Total size of the heap (ghost field for spec).
    total_size: Ghost<int>,
}
