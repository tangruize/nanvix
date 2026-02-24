    pub fn copy_from_user_unaligned(
        &self,
        dst: usize,
        src: usize,
        size: usize,
    ) -> (result: Result<(), Error>)
        requires
            self.inv(),
            // Source user pages must be mapped (original panics if not).
            size > 0 ==> self@.spec_user_region_is_mapped(src as int, size as int),
        ensures
            result.is_ok() ==> {
                &&& size > 0
                &&& spec_is_user_region(src as int, size as int)
                &&& spec_is_kernel_region(dst as int, size as int)
            },
    {
        // Check if size is zero.
        if size == 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "zero-length copy"));
        }

        // Check if source is in user space.
        if !Self::is_user_region(src, size) {
            return Err(Error::new(ErrorCode::BadAddress, "source not in user space"));
        }

        // Check if destination is in kernel space.
        if !Self::is_kernel_region(dst, size) {
            return Err(Error::new(ErrorCode::BadAddress, "destination not in kernel space"));
        }

        // Abstraction: The original performs a two-pass loop (dry-run then actual copy),
        // calling `find_user_frame()` per page to validate frame existence and obtain
        // physical addresses. This is replaced by the `spec_user_region_is_mapped`
        // precondition which captures the per-page mapping requirement at the spec level.
        // Limitation: The original's dry-run also implicitly validates that each source
        // frame's physical address is within bounds (via `__phys_memcpy`), but this model
        // does not postcondition on physical bounds of source frames. Physical bounds are
        // established at `map()` time by the allocator invariant (frames come from upool
        // which only allocates within MEMORY_SIZE).
        Ok(())
    }
