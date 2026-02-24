    fn try_from(raw_tid: u64) -> Result<Self, Self::Error> {
        raw_tid
            .try_into()
            .map_err(|_| Error::new(ErrorCode::InvalidArgument, "invalid thread identifier"))
            .map(ThreadIdentifier)
    }
