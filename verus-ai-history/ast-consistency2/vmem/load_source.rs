    pub fn load(&self) -> Result<(), Error> {
        let pgdir_addr: FrameAddress = self.pgdir.physical_address()?;
        unsafe { mmu::load_page_directory(pgdir_addr.into_raw_value()) };
        Ok(())
    }
