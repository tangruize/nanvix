    pub fn thread_state_mut(&mut self) -> &mut ThreadState {
        match self {
            ThreadRefMut::Ready(thread) => thread.thread_state_mut(),
            ThreadRefMut::Running(thread) => thread.thread_state_mut(),
            ThreadRefMut::Sleeping(thread) => thread.thread_state_mut(),
            ThreadRefMut::Interrupted(thread) => thread.thread_state_mut(),
            ThreadRefMut::Zombie(thread) => thread.thread_state_mut(),
        }
    }
