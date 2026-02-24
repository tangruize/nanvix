    pub fn from_raw_unchecked(addr: usize) -> (result: Self)
        requires
            spec_is_page_aligned(addr as int),
        ensures
            result.inv(),
            result.spec_addr() == addr as int,
    {
        PageAlignedAddr { addr }
    }
