    pub fn attach_pmio(&mut self, pid: ProcessIdentifier, port: AnyIoPort) -> Result<(), Error> {
        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
        let mut process: ProcessRefMut = pm.find_process_mut(pid)?;
        process.state_mut().add_pmio(port);
        Ok(())
    }
