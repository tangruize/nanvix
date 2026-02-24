pub struct PageMapping {
    /// Virtual address (page-aligned).
    vaddr: usize,
    /// Frame address (page-aligned).
    frame_addr: usize,
    /// Whether this entry is valid/in-use.
    valid: bool,
}
