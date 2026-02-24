    pub fn new(len: usize) -> Result<RawArray<T>, Error> {
        Ok(RawArray {
            storage: RawArrayStorage::new_managed(len)?,
        })
    }
