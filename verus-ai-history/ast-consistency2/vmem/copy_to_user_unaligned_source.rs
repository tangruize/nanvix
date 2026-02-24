    pub fn copy_to_user_unaligned(
        &self,
        dst: VirtualAddress,
        src: VirtualAddress,
        size: usize,
    ) -> Result<(), Error> {
        // Perform a dry run first to check for errors.
        self.copy_to_user_unaligned_unchecked(dst, src, size, true)?;
        // Perform the actual copy, this will panic on irrecoverable errors.
        self.copy_to_user_unaligned_unchecked(dst, src, size, false)
    }
