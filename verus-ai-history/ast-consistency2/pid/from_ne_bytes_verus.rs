    pub fn from_ne_bytes(bytes: [u8; 4]) -> (result: ProcessIdentifier)
        ensures
            result@.value == ProcessIdentifierView::from_ne_bytes_spec(bytes),
            result.inv(),
    {
        ProcessIdentifier { value: i32::from_ne_bytes(bytes) }
    }
