    pub fn new(id: usize) -> (result: Self)
        ensures
            !result@.locked,
            result@.is_unlocked(),
            result@ == SpinlockView::spec_new(id as nat),
            result@.id == id as nat,
            result.inv(),
    {
        proof { reveal(Spinlock::inv); }
        Spinlock { locked: false, id: id, token_issued: false }
    }
