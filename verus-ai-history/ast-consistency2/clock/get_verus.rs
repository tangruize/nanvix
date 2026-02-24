    pub fn get(&self) -> (result: (u32, u32))
        requires
            self.wf(),
            // Trust Boundary T1: the caller must establish that no concurrent
            // writer can modify major/minor between the two reads.
            Self::spec_no_concurrent_writer_assumption(),
        ensures
            self.spec_get_consistent(result.0, result.1),
            Self::spec_no_concurrent_writer_assumption(),
    {
        (self.major, self.minor)
    }
