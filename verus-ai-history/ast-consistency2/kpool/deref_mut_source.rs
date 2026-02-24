    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            core::slice::from_raw_parts_mut(self.base.into_raw_value() as *mut u8, mem::PAGE_SIZE)
        }
    }
