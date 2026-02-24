    pub fn from_raw_value(addr: usize) -> (result: Result<PageAlignedPhysAddr, Error>)
        ensures
            result is Ok ==> {
                let pa = result->Ok_0;
                &&& pa.inv()
                &&& pa.spec_raw_value() == addr as int
                &&& addr % FRAME_SIZE == 0
            },
            result is Err ==> addr % FRAME_SIZE != 0,
    {
        if addr % FRAME_SIZE != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "address not page-aligned"));
        }
        Ok(PageAlignedPhysAddr { raw_addr: addr })
    }
