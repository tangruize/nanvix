    pub fn memset(&mut self, dst: PageAligned<VirtualAddress>, value: u32) -> Result<(), Error> {
        // Get corresponding user page.
        let uframe: FrameAddress = self.find_user_frame(dst)?;
        let dst: PageAligned<PhysicalAddress> = uframe.into_physical_address();
        let base: *mut u8 = dst.into_raw_value() as *mut u8;

        // Safety: `base` points to a valid memory location and `mem::PAGE_SIZE` bytes are
        // writable.
        unsafe {
            __phys_memset(base, value as u8, mem::PAGE_SIZE);
        }

        Ok(())
    }
