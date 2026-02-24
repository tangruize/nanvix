pub struct KernelFrame {
    /// Frame address (page-aligned). Named `base` in the original.
    addr: FrameAddress,
    /// Pool identifier for provenance tracking.
    /// Replaces `Rc<RefCell<KpoolInner>>` from the original for pool association.
    pool_id: usize,
}
