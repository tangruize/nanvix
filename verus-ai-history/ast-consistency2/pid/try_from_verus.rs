    fn try_from(pid: ProcessIdentifier) -> Result<Self, Self::Error> {
        pid.try_into_u64()
    }
