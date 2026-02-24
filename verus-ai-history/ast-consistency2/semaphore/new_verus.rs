    pub fn new(value: usize) -> (result: Self)
        ensures
            result@ == Semaphore::spec_new_view(value as nat),
            result@.value == value as nat,
            result@.waiters == 0,
            result.wf(),
            result.spec_drop_safe(),
    {
        proof { reveal(Semaphore::wf); }
        Semaphore { value: value }
    }
