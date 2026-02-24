    pub fn from_raw(addr: usize) -> (result: Option<Self>)
        ensures
            // Liveness: alignment implies success.
            spec_is_page_aligned(addr as int) ==> result.is_some(),
            // Safety: success implies well-formed result.
            result.is_some() ==> {
                let pa = result.unwrap();
                &&& pa.inv()
                &&& pa.spec_addr() == addr as int
            },
            // Error case: failure implies misalignment.
            result.is_none() ==> !spec_is_page_aligned(addr as int),
    {
        if addr % PAGE_ALIGNMENT == 0 {
            Some(PageAlignedAddr { addr })
        } else {
            None
        }
    }
