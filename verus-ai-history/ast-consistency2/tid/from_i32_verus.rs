    pub fn from_i32(raw: i32) -> (result: ThreadIdentifier)
        ensures
            result@.value == raw as int,
            result@ == (ThreadIdentifierView { value: raw as int }),
            result.inv(),
    {
        ThreadIdentifier { value: raw }
    }
