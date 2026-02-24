    pub fn increment(&mut self) -> (result: u32)
        requires
            // Note: wf() is universally true for any (u32, u32) pair (see
            // lemma_always_wf). This precondition is retained for documentation
            // and forward-compatibility if wf() is ever strengthened.
            old(self).wf(),
        ensures
            self.wf(),
            old(self)@.is_max() ==> self@.ticks == 0,
            !old(self)@.is_max() ==> self@.ticks == old(self)@.ticks + 1,
            self@.ticks == old(self)@.next_ticks(),
    {
        if self.minor < u32::MAX {
            self.minor = self.minor + 1;
            proof {
                assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                assert(old(self).spec_minor() < u32::MAX as nat);
                assert(old(self).spec_major() <= u32::MAX as nat);
                Self::lemma_nat_mul_le_mono(
                    old(self).spec_major(), u32::MAX as nat, Self::MINOR_MODULUS(),
                );
                assert(!old(self).spec_is_max());
                // self.major unchanged, self.minor = old(self).minor + 1.
                assert(self.spec_major() == old(self).spec_major());
                assert(self.spec_minor() == old(self).spec_minor() + 1);
                // spec_ticks = major * M + minor = old.major * M + (old.minor + 1).
                assert(self.spec_ticks() == old(self).spec_major() * Self::MINOR_MODULUS() + old(self).spec_minor() + 1);
                self.lemma_always_wf();
            }
        } else {
            self.minor = 0;
            if self.major < u32::MAX {
                self.major = self.major + 1;
                proof {
                    assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
                    assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                    assert(old(self).spec_major() < u32::MAX as nat);
                    Self::lemma_nat_mul_le_mono(
                        old(self).spec_major(), (u32::MAX - 1) as nat, Self::MINOR_MODULUS(),
                    );
                    assert(old(self).spec_ticks() < u64::MAX as nat);
                    assert(!old(self).spec_is_max());
                    // self.major = old.major + 1, self.minor = 0.
                    let om: nat = old(self).spec_major();
                    let m: nat = Self::MINOR_MODULUS();
                    assert((om + 1) * m == om * m + m) by(nonlinear_arith);
                    self.lemma_always_wf();
                }
            } else {
                self.major = 0;
                proof {
                    assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
                    assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                    assert(old(self).spec_ticks() == u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat);
                    assert(old(self).spec_is_max());
                    assert(self.spec_ticks() == 0);
                    assert(self.spec_ticks() == old(self).spec_next_ticks());
                    self.lemma_always_wf();
                }
            }
        }
        self.minor
    }
