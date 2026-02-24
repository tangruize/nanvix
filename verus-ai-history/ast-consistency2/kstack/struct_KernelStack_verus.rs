pub struct KernelStack {
    /// Base virtual address of the stack.
    base_addr: usize,
    /// Number of pages in the stack.
    num_pages: usize,
}
