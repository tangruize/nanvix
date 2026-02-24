    fn new() -> (ReadyThread, Self) {
        let kernel: ReadyThread = {
            ReadyThread::new(
                From::<i32>::from(0),
                None,
                None,
                None,
                ContextInformation::default(),
                // SAFETY: calls to FpuState::new are synchronized.
                unsafe { FpuState::new() },
            )
        };
        (
            kernel,
            Self {
                next_id: From::<i32>::from(1),
            },
        )
    }
