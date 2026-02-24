    fn put_cond(&mut self, cond_addr: ConditionAddress) -> Result<(), Error> {
        self.get_running_mut().state_mut().put_cond(cond_addr)
    }
