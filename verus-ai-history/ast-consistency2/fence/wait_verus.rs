    pub fn wait(&self)
        requires
            self.inv(),
            self@.is_satisfied(),
        ensures
            self@.is_satisfied(),
    {
        // In the sequential model, the precondition guarantees satisfaction,
        // so the spin loop body is never entered.
    }
