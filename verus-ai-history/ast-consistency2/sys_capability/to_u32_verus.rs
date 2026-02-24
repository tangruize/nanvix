    pub fn to_u32(&self) -> (result: u32)
        requires
            self.inv(),
        ensures
            result as int == self@.value,
            CapabilityView::is_valid_discriminant(result as int),
    {
        match *self {
            Capability::ExceptionControl => 0u32,
            Capability::InterruptControl => 1u32,
            Capability::IoManagement => 2u32,
            Capability::MemoryManagement => 3u32,
            Capability::ProcessManagement => 4u32,
        }
    }
