    pub fn drop_check(&self)
        requires
            self.wf(),
            self@.spec_drop_safe(),
        ensures
            self@.spec_is_empty(),
    {
    }
