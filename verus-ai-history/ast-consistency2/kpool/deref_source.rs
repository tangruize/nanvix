    fn deref(&self) -> &Self::Target {
        unsafe {
            core::slice::from_raw_parts(self.base.into_raw_value() as *const u8, mem::PAGE_SIZE)
        }
    }
