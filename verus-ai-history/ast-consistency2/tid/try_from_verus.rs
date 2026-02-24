    fn try_from(tid: ThreadIdentifier) -> Result<Self, Self::Error> {
        tid.try_into_u64()
    }
