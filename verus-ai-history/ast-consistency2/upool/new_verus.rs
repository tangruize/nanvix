    pub fn new(frame_allocator: FrameAllocator) -> (result: Upool)
        requires
            frame_allocator.inv(),
        ensures
            result.inv(),
            result@.capacity() == frame_allocator@.capacity,
            // Allocated set is preserved.
            forall|i: int| 0 <= i < result@.capacity() ==>
                result@.is_allocated(i) == frame_allocator@.is_allocated(i),
            // Fresh initialization is preserved (bidirectional).
            result@.is_freshly_initialized() <==> frame_allocator@.is_freshly_initialized(),
    {
        Upool { frame_allocator }
    }
