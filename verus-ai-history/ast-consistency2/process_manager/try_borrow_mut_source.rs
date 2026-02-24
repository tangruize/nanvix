    fn try_borrow_mut(&mut self) -> Result<RefMut<'_, ProcessManagerInner>, Error> {
        match self.0.try_borrow_mut() {
            Ok(pm) => Ok(pm),
            Err(_) => {
                let reason: &str = "cannot borrow process manager";
                error!("{reason}");
                Err(Error::new(ErrorCode::ResourceBusy, reason))
            },
        }
    }
