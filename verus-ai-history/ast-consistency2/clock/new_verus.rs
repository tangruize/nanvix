    pub fn new() -> (result: Self)
        ensures
            result@.ticks == 0,
            result@.is_zero(),
            result@ == TimerTicks::spec_new_view(),
            result.wf(),
    {
        let r = TimerTicks { minor: 0, major: 0 };
        proof { r.lemma_always_wf(); }
        r
    }
