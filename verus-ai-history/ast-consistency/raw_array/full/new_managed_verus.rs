    fn new_managed(len: usize) -> Result<RawArrayStorage<T>, Error> {
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid length"));
        }
        let layout: Layout = match Layout::array::<T>(len) {
            Ok(layout) => layout,
            Err(_) => return Err(Error::new(ErrorCode::InvalidArgument, "invalid layout")),
        };
        let ptr: ptr::NonNull<T> = {
            let ptr: *mut u8 = unsafe { alloc(layout) };
            match ptr::NonNull::new(ptr as *mut T) {
                Some(p) => p,
                None => return Err(Error::new(ErrorCode::OutOfMemory, "out of memory")),
            }
        };
        unsafe { ptr::write_bytes(ptr.as_ptr(), 0, len) };
        Ok(RawArrayStorage::Managed { ptr, len })
    }
