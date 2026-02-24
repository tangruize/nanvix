    pub fn mmio_alloc(
        &mut self,
        pid: ProcessIdentifier,
        region: IoMemoryRegion,
    ) -> Result<(), Error> {
        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
        let mut process: ProcessRefMut = pm.find_process_mut(pid)?;
        let state: &mut ProcessState = process.state_mut();

        // TODO: change page permissions.
        let vmem: &mut Vmem = state.vmem_mut();
        vmem.kctrl(region.base(), region.perm())?;

        state.add_mmio(region);

        Ok(())
    }
