    pub fn is_max(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self@.is_max(),
    {
        proof {
            Self::lemma_max_ticks_value();
        }
        self.minor == u32::MAX && self.major == u32::MAX
    }
