    fn try_from(value: i64) -> Result<Self, Self::Error> {
        // Attempt to convert i64 to i32.
        let value: i32 = value
            .try_into()
            .map_err(|_| Error::new(ErrorCode::InvalidArgument, "invalid error code"))?;

        // Attempt to convert i32 to ErrorCode.
        ErrorCode::try_from(value)
            .map_err(|_| Error::new(ErrorCode::InvalidArgument, "invalid error code"))
    }
