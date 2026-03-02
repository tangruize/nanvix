    pub fn is_power_of_two(n: usize) -> (result: bool)
        requires n > 0,
        ensures result == Self::spec_is_power_of_two(n as int),
    {
        let mut val: usize = n;

        // Loop invariant: val > 0 and the result depends on whether val becomes 1.
        // We divide by 2 as long as val is even and > 1.
        while val > 1 && val % 2 == 0
            invariant
                val > 0,
                Self::spec_is_power_of_two(n as int) == Self::spec_is_power_of_two(val as int),
            decreases val,
        {
            val = val / 2;
        }

        // At this point: either val == 1 (power of two) or val > 1 && val % 2 != 0 (not power of two).
        val == 1
    }
