    pub fn new(mm: &mut VirtMemoryManager) -> Result<Self, Error> {
        let kpages: Vec<KernelPage> =
            mm.alloc_kpages(true, config::kernel::KSTACK_SIZE / ::arch::mem::PAGE_SIZE)?;

        Ok(Self { kpages })
    }
