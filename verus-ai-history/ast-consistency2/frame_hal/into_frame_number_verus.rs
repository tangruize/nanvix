    pub fn into_frame_number(self) -> (result: FrameNumber)
        requires
            self.inv(),
            // The original ensures this via PhysicalAddress bounds checking.
            self.spec_raw_value() / FRAME_SIZE as int <= MAX_FRAME_NUMBER as int,
        ensures
            result.inv(),
            result.spec_raw_value() == self.spec_frame_number(),
    {
        FrameNumber { value: self.raw_addr / FRAME_SIZE }
    }
