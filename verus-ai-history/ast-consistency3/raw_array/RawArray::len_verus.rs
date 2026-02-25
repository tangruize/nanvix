    pub fn len(&self) -> (result: usize)
        ensures
            result == self@.len(),
    {
        match &self.storage {
            RawArrayStorage::Managed { len, .. } => *len,
            RawArrayStorage::Unmanaged { len, .. } => *len,
        }
    }
