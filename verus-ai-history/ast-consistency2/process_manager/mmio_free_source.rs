    pub fn mmio_free(
        &mut self,
        pid: ProcessIdentifier,
        addr: PageAligned<VirtualAddress>,
    ) -> Result<(), Error> {
        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
        let mut process: ProcessRefMut = pm.find_process_mut(pid)?;
        let state: &mut ProcessState = process.state_mut();
        state.remove_mmio(addr);

        Ok(())
    }
