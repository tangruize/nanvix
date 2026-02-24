    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Capability::ExceptionControl),
            1 => Ok(Capability::InterruptControl),
            2 => Ok(Capability::IoManagement),
            3 => Ok(Capability::MemoryManagement),
            4 => Ok(Capability::ProcessManagement),
            _ => Err(Error::new(ErrorCode::InvalidArgument, "invalid capability")),
        }
    }
