    unsafe fn new_unmanaged(ptr: *mut T, len: usize) -> Result<RawArrayStorage<T>, Error> {
        // Check if the length is invalid.
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid length"));
        }

        // Check if memory region wraps around.
        if ptr.wrapping_add(len) < ptr {
            return Err(Error::new(ErrorCode::InvalidArgument, "wrapping memory region"));
        }

        // Check and cast provided slice.
        let ptr: ptr::NonNull<T> = match ptr::NonNull::new(ptr) {
            Some(ptr) => ptr,
            None => return Err(Error::new(ErrorCode::InvalidArgument, "invalid pointer")),
        };

        // Initialize the backing storage.
        ptr::write_bytes(ptr.as_ptr(), 0, len);

        Ok(RawArrayStorage::Unmanaged { ptr, len })
    }
