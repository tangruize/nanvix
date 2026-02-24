    pub fn remove_pmio(&mut self, port_number: u16) -> Result<AnyIoPort, Error> {
        let index: Option<usize> = self.pmio.iter().position(|p| p.number() == port_number);
        match index {
            Some(index) => Ok(self.pmio.remove(index)),
            None => {
                let reason: &'static str = "io port not found";
                error!("{:?}", reason);
                Err(Error::new(ErrorCode::NoSuchEntry, reason))
            },
        }
    }
