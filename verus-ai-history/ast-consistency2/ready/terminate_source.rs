    pub fn terminate(self) -> ZombieThread {
        ZombieThread::from_state(self.state, ErrorCode::Interrupted.into())
    }
