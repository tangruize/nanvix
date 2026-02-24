    pub fn try_from_i64(raw: i64) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& ProcessIdentifierView::in_i32_range(raw as int)
                &&& result->Ok_0@.value == raw as int
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !ProcessIdentifierView::in_i32_range(raw as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if raw < i32::MIN as i64 || raw > i32::MAX as i64 {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
        }
        Ok(ProcessIdentifier { value: raw as i32 })
    }
