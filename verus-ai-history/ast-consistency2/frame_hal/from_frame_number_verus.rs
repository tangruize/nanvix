    pub fn from_frame_number(frame_number: FrameNumber) -> (result: Result<FrameAddress, Error>)
        requires
            frame_number.inv(),
            frame_number.spec_raw_value() <= MAX_FRAME_NUMBER as int,
        ensures
            // Always succeeds when precondition is met.
            result is Ok,
            result is Ok ==> {
                let addr = result->Ok_0;
                &&& addr.inv()
                &&& addr.spec_frame_number() == frame_number.spec_raw_value()
                &&& addr.spec_is_aligned()
                &&& addr.spec_raw_value() == frame_number.spec_raw_value() * FRAME_SIZE as int
            },
    {
        let raw_value: usize = frame_number.into_raw_value();
        // Check for overflow: raw_value * FRAME_SIZE must fit in usize.
        // Since raw_value <= MAX_FRAME_NUMBER = usize::MAX / FRAME_SIZE (on 32-bit),
        // this check always passes when precondition is satisfied.
        if raw_value > usize::MAX / FRAME_SIZE {
            return Err(Error::new(ErrorCode::InvalidArgument, "frame number overflow"));
        }
        let addr: usize = raw_value * FRAME_SIZE;
        Ok(FrameAddress { raw_addr: addr })
    }
