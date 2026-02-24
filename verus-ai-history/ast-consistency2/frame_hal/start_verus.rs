    pub fn start(&self) -> (result: PageAlignedPhysAddr)
        requires self.inv(),
        ensures
            result.inv(),
            result.spec_raw_value() == self.spec_start(),
            result.spec_frame_number() == self.spec_start_frame(),
    {
        proof {
            // Reveal that spec_start_frame() == start.spec_frame_number().
            // This is trivially true by definition.
        }
        self.start
    }
