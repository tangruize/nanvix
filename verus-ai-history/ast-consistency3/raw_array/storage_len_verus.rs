    fn storage_len(&self) -> usize {
        match self {
            RawArrayStorage::Managed { len, .. } => *len,
            RawArrayStorage::Unmanaged { len, .. } => *len,
        }
    }
