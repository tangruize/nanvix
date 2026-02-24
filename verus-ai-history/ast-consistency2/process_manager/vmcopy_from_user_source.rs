    pub fn vmcopy_from_user(
        &mut self,
        pid: ProcessIdentifier,
        dst: VirtualAddress,
        src: VirtualAddress,
        size: usize,
    ) -> Result<(), Error> {
        self.try_borrow_mut()?
            .find_process_mut(pid)?
            .state_mut()
            .copy_from_user_unaligned(dst, src, size)
    }
