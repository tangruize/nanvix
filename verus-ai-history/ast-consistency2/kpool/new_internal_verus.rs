    fn new_internal(addr: FrameAddress, pool_id: usize) -> (result: KernelFrame)
        requires
            addr.spec_is_aligned(),
        ensures
            result.inv(),
            result@.frame_number == addr.spec_frame_number(),
            result@.pool_id == pool_id as int,
            // Backward-compatible accessors.
            result.spec_address() == addr,
            result.spec_is_aligned(),
            result.spec_frame_number() == addr.spec_frame_number(),
            result.spec_pool_id() == pool_id as int,
    {
        KernelFrame { addr, pool_id }
    }
