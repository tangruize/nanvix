    fn is_kernel_addr(vaddr: usize) -> (result: bool)
        ensures
            result == spec_is_kernel_addr(vaddr as int),
    {
        !Self::is_user_addr(vaddr)
    }
