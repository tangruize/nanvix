    fn get_pmio(&self, port_number: u16) -> Result<&AnyIoPort, Error> {
        let port: Option<&AnyIoPort> = self.pmio.iter().find(|p| p.number() == port_number);
        match port {
            Some(port) => Ok(port),
            None => {
                let reason: &'static str = "io port not found";
                error!("{:?}", reason);
                Err(Error::new(ErrorCode::NoSuchEntry, reason))
            },
        }
    }
