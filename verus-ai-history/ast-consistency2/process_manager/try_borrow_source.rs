    fn try_borrow(&self) -> Result<Ref<'_, ProcessManagerInner>, Error> {
        match self.0.try_borrow() {
            Ok(pm) => Ok(pm),
            Err(_) => {
                let reason: &str = "cannot borrow process manager";
                error!("{reason}");
                Err(Error::new(ErrorCode::ResourceBusy, reason))
            },
        }
    }
