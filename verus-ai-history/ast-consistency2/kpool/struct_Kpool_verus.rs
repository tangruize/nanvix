pub struct Kpool {
    /// Underlying frame allocator.
    frame_allocator: FrameAllocator,
    /// Pool identifier for provenance tracking.
    pool_id: usize,
}
