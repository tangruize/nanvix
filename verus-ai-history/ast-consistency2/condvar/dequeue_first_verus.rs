    pub fn dequeue_first(&mut self) -> (dequeued: bool)
        requires
            old(self).wf(),
        ensures
            dequeued == !old(self)@.spec_is_empty(),
            dequeued ==> self@.spec_len() == old(self)@.spec_len() - 1,
            dequeued ==> self@.sleeping =~= old(self)@.sleeping.subrange(
                1,
                old(self)@.sleeping.len() as int,
            ),
            // When dequeued, the removed entry was the front of the queue.
            dequeued ==> old(self)@.sleeping[0] == old(self)@.spec_front(),
            !dequeued ==> self@ == old(self)@,
            self.wf(),
    {
        if self.len > 0 {
            self.len = self.len - 1;
            let _removed: (i32, i32) = self.sleeping.remove(0);
            true
        } else {
            false
        }
    }
