    pub fn from_ne_bytes(bytes: [u8; 4]) -> (result: ThreadIdentifier)
        ensures
            result@.value == ThreadIdentifierView::from_ne_bytes_spec(bytes),
            result.inv(),
    {
        ThreadIdentifier { value: i32::from_ne_bytes(bytes) }
    }
