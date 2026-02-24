    pub fn vmcopy_to_user(
        &mut self,
        pid: ProcessIdentifier,
        dst: VirtualAddress,
        src: VirtualAddress,
        size: usize,
    ) -> Result<(), Error> {
        self.try_borrow_mut()?
            .find_process_mut(pid)?
            .state_mut()
            .copy_to_user_unaligned(dst, src, size)
    }
