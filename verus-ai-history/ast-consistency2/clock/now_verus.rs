    pub fn now(&self, timer_freq: u32) -> (result: (u64, u32))
        requires
            timer_freq > 0,
            self.wf(),
            Self::spec_no_concurrent_writer_assumption(),
        ensures
            result.0 as nat == self@.ticks / timer_freq as nat,
            result.1 as nat == self@.nanoseconds(timer_freq as nat),
            Self::spec_nanoseconds_valid(result.1 as nat),
            result.1 < 1_000_000_000u32,
    {
        let (major_ticks, minor_ticks): (u32, u32) = self.get();
        let seconds: u64 = Self::compute_seconds(major_ticks, minor_ticks, timer_freq);
        let nanoseconds: u32 = Self::compute_nanoseconds(minor_ticks, timer_freq);
        proof {
            // Prove minor_ticks == self@.ticks % MINOR_MODULUS (for nanoseconds equivalence).
            let m: nat = Self::MINOR_MODULUS();
            let minor_nat: nat = minor_ticks as nat;
            let major_nat: nat = major_ticks as nat;
            assert(minor_nat < m);
            assert(major_nat * m + minor_nat == self.spec_ticks());
            assert((major_nat * m + minor_nat) % m == minor_nat) by(nonlinear_arith)
                requires(minor_nat < m && m > 0);
        }
        (seconds, nanoseconds)
    }
