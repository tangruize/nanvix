    pub fn is_zero(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self@.is_zero(),
    {
        proof {
            assert(Self::MINOR_MODULUS() > 0);
            if self.major > 0 {
                assert(self.spec_major() >= 1);
                assert(self.spec_major() * Self::MINOR_MODULUS() >= Self::MINOR_MODULUS());
                assert(self.spec_ticks() >= Self::MINOR_MODULUS());
            }
        }
        self.minor == 0 && self.major == 0
    }
