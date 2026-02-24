    fn get_pmio_mut(&mut self, port_number: u16) -> Result<&mut AnyIoPort, Error> {
        let port: Option<&mut AnyIoPort> = self.pmio.iter_mut().find(|p| p.number() == port_number);
        match port {
            Some(port) => Ok(port),
            None => {
                let reason: &'static str = "io port not found";
                error!("{:?}", reason);
                Err(Error::new(ErrorCode::NoSuchEntry, reason))
            },
        }
    }
