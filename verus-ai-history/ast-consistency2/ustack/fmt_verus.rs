    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "UserStack {{ base: {:?}, top: {:?}, size={:?} }}",
            self.base_addr,
            self.base_addr + USER_STACK_SIZE,
            USER_STACK_SIZE
        )
    }
