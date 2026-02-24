    pub fn is_user_addr(vaddr: usize) -> (result: bool)
        ensures
            result == spec_is_user_addr(vaddr as int),
    {
        vaddr >= USER_BASE && vaddr < USER_END
    }
