    pub fn new(id: usize) -> (result: Self)
        ensures
            !result@.locked,
            result@.is_unlocked(),
            result@ == MutexView::spec_new(id as nat),
            result@.id == id as nat,
            result.wf(),
    {
        proof { reveal(Mutex::wf); }
        Mutex { locked: false, id: id, token_issued: false }
    }
