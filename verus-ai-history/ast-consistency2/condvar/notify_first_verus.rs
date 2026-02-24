    pub fn notify_first(&mut self) -> (awakened: u32)
        requires
            old(self).wf(),
        ensures
            self.wf(),
            awakened <= 1,
            old(self)@.spec_is_empty() ==> (awakened == 0 && self@ == old(self)@),
            !old(self)@.spec_is_empty() ==> (
                awakened == 1
                && self@.spec_len() == old(self)@.spec_len() - 1
            ),
    {
        if self.dequeue_first() {
            1u32
        } else {
            0u32
        }
    }
