    pub fn try_from_u32(value: u32) -> (result: Result<Capability, Error>)
        ensures
            result is Ok ==> {
                &&& CapabilityView::is_valid_discriminant(value as int)
                &&& result->Ok_0@.value == value as int
                &&& result->Ok_0 == CapabilityView::from_discriminant(value as int)
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !CapabilityView::is_valid_discriminant(value as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        match value {
            0u32 => Ok(Capability::ExceptionControl),
            1u32 => Ok(Capability::InterruptControl),
            2u32 => Ok(Capability::IoManagement),
            3u32 => Ok(Capability::MemoryManagement),
            4u32 => Ok(Capability::ProcessManagement),
            _ => Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE)),
        }
    }
