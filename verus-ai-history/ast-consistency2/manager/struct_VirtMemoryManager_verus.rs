pub struct VirtMemoryManager {
    /// Kernel frame pool for page tables and kernel pages.
    kpool: Kpool,
    /// User frame pool for user pages.
    upool: Upool,
}
