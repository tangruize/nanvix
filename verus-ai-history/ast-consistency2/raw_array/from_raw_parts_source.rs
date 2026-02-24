    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> Result<RawArray<T>, Error> {
        Ok(RawArray {
            storage: RawArrayStorage::new_unmanaged(ptr, len)?,
        })
    }
