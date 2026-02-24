    pub fn mctrl(
        &mut self,
        mm: &mut VirtMemoryManager,
        pid: ProcessIdentifier,
        vaddr: PageAligned<VirtualAddress>,
        access: AccessPermission,
    ) -> Result<(), Error> {
        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
        let mut process: ProcessRefMut = pm.find_process_mut(pid)?;
        let vmem: &mut Vmem = process.state_mut().vmem_mut();
        mm.ctrl_upage(vmem, vaddr, access)
    }
