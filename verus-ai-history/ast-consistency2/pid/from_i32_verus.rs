    pub fn from_i32(raw: i32) -> (result: ProcessIdentifier)
        ensures
            result@.value == raw as int,
            result@ == (ProcessIdentifierView { value: raw as int }),
            result.inv(),
    {
        ProcessIdentifier { value: raw }
    }
