    pub fn compute_nanoseconds(minor_ticks: u32, timer_freq: u32) -> (result: u32)
        requires
            timer_freq > 0,
        ensures
            result as nat == Self::spec_compute_nanoseconds(minor_ticks, timer_freq),
            Self::spec_nanoseconds_valid(result as nat),
            result < 1_000_000_000u32,
    {
        proof {
            Self::lemma_nanoseconds_in_range(minor_ticks, timer_freq);
        }
        (minor_ticks % timer_freq) * (1_000_000_000u32 / timer_freq)
    }
