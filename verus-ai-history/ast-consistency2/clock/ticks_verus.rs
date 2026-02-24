    pub fn ticks(&self) -> (result: u64)
        requires
            self.wf(),
        ensures
            result as nat == self@.ticks,
    {
        proof {
            assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        }
        (self.major as u64) * 0x1_0000_0000u64 + (self.minor as u64)
    }
