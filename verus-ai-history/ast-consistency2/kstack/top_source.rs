    pub fn top(&self) -> PageAligned<VirtualAddress> {
        let base: usize = self.kpages[0].base().into_raw_value();
        let size: usize = config::kernel::KSTACK_SIZE;
        // SAFETY: The following call to unwrap is safe because the base address of the kernel stack
        // and the size of the kernel stack are both page aligned.
        debug_assert!(::sys::mm::is_aligned(base, PAGE_ALIGNMENT));
        debug_assert!(::sys::mm::is_aligned(size, PAGE_ALIGNMENT));
        PageAligned::from_raw_value(base + size).unwrap()
    }
