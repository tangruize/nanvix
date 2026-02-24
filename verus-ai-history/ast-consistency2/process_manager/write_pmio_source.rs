    pub fn write_pmio(
        &mut self,
        pid: ProcessIdentifier,
        port_number: u16,
        port_width: IoPortWidth,
        value: u32,
    ) -> Result<(), Error> {
        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
        let mut process: ProcessRefMut = pm.find_process_mut(pid)?;
        process
            .state_mut()
            .write_pmio(port_number, port_width, value)
    }
