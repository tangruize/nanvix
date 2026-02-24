    pub fn remove_mmio(&mut self, addr: PageAligned<VirtualAddress>) {
        self.mmio.retain(|r| r.base() != addr)
    }
