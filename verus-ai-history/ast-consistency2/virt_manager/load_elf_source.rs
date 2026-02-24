    pub fn load_elf(
        &mut self,
        vmem: &mut Vmem,
        elf: &Elf32Fhdr,
    ) -> Result<(VirtualAddress, PageAligned<VirtualAddress>), Error> {
        elf::elf32_load(self, vmem, elf)
    }
