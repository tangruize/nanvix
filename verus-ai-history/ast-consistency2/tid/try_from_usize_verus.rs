    pub fn try_from_usize(raw: usize) -> (result: Result<ThreadIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& ThreadIdentifierView::in_non_negative_i32_range(raw as int)
                &&& result->Ok_0@.value == raw as int
                &&& result->Ok_0@.is_non_negative()
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !ThreadIdentifierView::in_non_negative_i32_range(raw as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if raw > i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
        }
        Ok(ThreadIdentifier { value: raw as i32 })
    }
