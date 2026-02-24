struct KpoolInner {
    /// Size of the kernel pool.
    region: TruncatedMemoryRegion<PhysicalAddress>,
    /// Bitmap of free pages.
    bitmap: Bitmap,
}
