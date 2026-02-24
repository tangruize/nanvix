    pub fn notify_all(&mut self) -> (count: usize)
        requires
            old(self).wf(),
        ensures
            self.wf(),
            count as nat == old(self)@.spec_len(),
            self@.spec_is_empty(),
            self@ == CondvarView::spec_new(),
    {
        self.clear()
    }
