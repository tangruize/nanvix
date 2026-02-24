    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "KernelStack {{ base: {:?}, top: {:?}, size={:?} }}",
            self.base(),
            self.top(),
            self.size()
        )
    }
