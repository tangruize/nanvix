    fn MUTEX_MAX_EXEC() -> (result: usize)
        ensures
            result == Self::MUTEX_MAX(),
    {
        32usize
    }
