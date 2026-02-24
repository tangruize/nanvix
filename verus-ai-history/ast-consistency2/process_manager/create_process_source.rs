    pub fn create_process(
        &mut self,
        mm: &mut VirtMemoryManager,
        elf: &Elf32Fhdr,
        args: &str,
        env: &str,
    ) -> Result<ProcessIdentifier, Error> {
        self.try_borrow_mut()?.create_process(mm, elf, args, env)
    }
