    fn get_cond(&mut self, cond_addr: ConditionAddress) -> Result<Condvar, Error> {
        self.get_running_mut().state_mut().get_cond(cond_addr)
    }
