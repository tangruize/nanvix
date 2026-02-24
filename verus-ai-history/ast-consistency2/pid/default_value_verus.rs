    pub fn default_value() -> (result: ProcessIdentifier)
        ensures
            result@.value == 0,
            result@.is_kernel(),
            result.inv(),
    {
        ProcessIdentifier { value: 0 }
    }
