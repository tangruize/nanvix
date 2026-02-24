    pub fn detach_pmio(
        &mut self,
        pid: ProcessIdentifier,
        port_number: u16,
    ) -> Result<AnyIoPort, Error> {
        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
        let mut process: ProcessRefMut = pm.find_process_mut(pid)?;
        process.state_mut().remove_pmio(port_number)
    }
