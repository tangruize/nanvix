    pub fn new_vmem(&self, vmem: &Vmem) -> (result: Vmem)
        requires
            self.inv(),
            vmem.inv(),
        ensures
            self.inv(),
            result.inv(),
            // NOTE: Uses view (result@.mapping_count) per Vmem spec methodology.
            result@.mapping_count == 0,
    {
        Vmem::clone(vmem)
    }
