    pub fn default_value() -> (result: ThreadIdentifier)
        ensures
            result@.value == 0,
            result@.is_kernel(),
            result.inv(),
    {
        ThreadIdentifier { value: 0 }
    }
