    pub fn try_into_u64(self) -> (result: Result<u64, Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                &&& self@.is_non_negative()
                &&& result->Ok_0 as int == self@.value
            },
            result is Err ==> {
                &&& !self@.is_non_negative()
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if self.value < 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
        }
        Ok(self.value as u64)
    }
