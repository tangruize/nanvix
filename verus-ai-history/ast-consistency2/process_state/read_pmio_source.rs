    pub fn read_pmio(&self, port_number: u16, port_width: IoPortWidth) -> Result<u32, Error> {
        let port: &AnyIoPort = self.get_pmio(port_number)?;
        port.read(port_width)
    }
