    pub fn try_from_isize(raw: isize) -> (result: Result<ThreadIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& ThreadIdentifierView::in_i32_range(raw as int)
                &&& result->Ok_0@.value == raw as int
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !ThreadIdentifierView::in_i32_range(raw as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if raw < i32::MIN as isize || raw > i32::MAX as isize {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
        }
        Ok(ThreadIdentifier { value: raw as i32 })
    }
