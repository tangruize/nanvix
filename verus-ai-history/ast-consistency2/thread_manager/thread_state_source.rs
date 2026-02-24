    pub fn thread_state(&self) -> &ThreadState {
        match self {
            ThreadRef::Ready(thread) => thread.thread_state(),
            ThreadRef::Running(thread) => thread.thread_state(),
            ThreadRef::Sleeping(thread) => thread.thread_state(),
            ThreadRef::Interrupted(thread) => thread.thread_state(),
            ThreadRef::Zombie(thread) => thread.thread_state(),
        }
    }
