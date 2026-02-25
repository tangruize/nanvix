    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value: i32 = value
            .try_into()
            .map_err(|_| Error::new(ErrorCode::InvalidArgument, "invalid error code"))?;
        ErrorCode::try_from(value)
            .map_err(|_| Error::new(ErrorCode::InvalidArgument, "invalid error code"))
    }
