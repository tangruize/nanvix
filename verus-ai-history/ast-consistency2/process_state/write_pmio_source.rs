    pub fn write_pmio(
        &mut self,
        port_number: u16,
        port_width: IoPortWidth,
        value: u32,
    ) -> Result<(), Error> {
        let port: &mut AnyIoPort = self.get_pmio_mut(port_number)?;
        port.write(port_width, value)
    }
