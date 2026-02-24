    pub fn new_vmem(&self, vmem: &Vmem) -> Result<Vmem, Error> {
        let new_vmem: Vmem = Vmem::clone(vmem)?;

        trace!(
            "new_vmem={:?}, old_vmem={:?}",
            new_vmem.pgdir().physical_address(),
            vmem.pgdir().physical_address()
        );

        Ok(new_vmem)
    }
