    pub fn compute_seconds(major_ticks: u32, minor_ticks: u32, timer_freq: u32) -> (result: u64)
        requires
            timer_freq > 0,
        ensures
            result as nat == Self::spec_compute_seconds(major_ticks, minor_ticks, timer_freq),
    {
        proof {
            assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        }
        let total_ticks: u64 = (major_ticks as u64) * 0x1_0000_0000u64 + (minor_ticks as u64);
        total_ticks / (timer_freq as u64)
    }
