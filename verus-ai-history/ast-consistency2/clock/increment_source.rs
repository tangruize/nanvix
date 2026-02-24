    fn increment(&self) -> u32 {
        // Safely increment `TIMER_TICKS`, assuming single-writer.
        let minor: u32 = self.minor.load(ORDER);
        let new_minor: u32 = minor.wrapping_add(1);
        self.minor.store(new_minor, ORDER);

        // Check if the minor tick overflowed.
        if new_minor == 0 {
            // Safely increment `TIMER_TICKS_MAJOR`, assuming single-writer.
            let major: u32 = self.major.load(ORDER);
            self.major.store(major.wrapping_add(1), ORDER);
        }

        new_minor
    }
