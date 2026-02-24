    pub fn from_frame_number(frame_number: FrameNumber) -> Result<Self, Error> {
        Ok(Self(PageAligned::from_address(PhysicalAddress::from_number(frame_number))?))
    }
