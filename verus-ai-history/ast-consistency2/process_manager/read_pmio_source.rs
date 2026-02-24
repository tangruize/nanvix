    pub fn read_pmio(
        &mut self,
        pid: ProcessIdentifier,
        port_number: u16,
        port_width: IoPortWidth,
    ) -> Result<u32, Error> {
        let pm: Ref<ProcessManagerInner> = self.try_borrow()?;
        let process: ProcessRef = pm.find_process(pid)?;
        process.state().read_pmio(port_number, port_width)
    }
