    pub fn new() -> (result: Self)
        ensures
            result@.spec_is_empty(),
            result@ == CondvarView::spec_new(),
            result.wf(),
    {
        Condvar { len: 0, sleeping: Vec::new() }
    }
