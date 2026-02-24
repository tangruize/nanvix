    pub fn clear(&mut self) -> (count: usize)
        requires
            old(self).wf(),
        ensures
            count as nat == old(self)@.spec_len(),
            // Any actual awakened count from the original satisfies this.
            CondvarView::spec_notify_all_result(0, count as nat),
            CondvarView::spec_notify_all_result(count as nat, count as nat),
            self@.spec_is_empty(),
            self@ == CondvarView::spec_new(),
            self.wf(),
    {
        let old_len: usize = self.len;
        self.len = 0;
        self.sleeping.clear();
        old_len
    }
