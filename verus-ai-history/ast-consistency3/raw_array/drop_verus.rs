    fn drop(&mut self) {
        match self {
            RawArrayStorage::Managed { ptr, len } => {
                if let Ok(layout) = Layout::array::<T>(*len) {
                    unsafe { dealloc(ptr.as_ptr() as *mut u8, layout); }
                }
            },
            RawArrayStorage::Unmanaged { .. } => (),
        }
    }
