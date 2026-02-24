    pub fn wait(
        &mut self,
        pid_val: i32,
        tid_val: i32,
        alarm_expired: bool,
    ) -> (enqueued: bool)
        requires
            old(self).wf(),
            old(self)@.spec_len() < usize::MAX,
            !old(self)@.spec_contains_entry(pid_val as int, tid_val as int),
            pid_val as int != CondvarView::spec_kernel_pid(),
        ensures
            enqueued == !alarm_expired,
            enqueued ==> self@.spec_len() == old(self)@.spec_len() + 1,
            enqueued ==> self@.sleeping =~= old(self)@.sleeping.push(
                (pid_val as int, tid_val as int),
            ),
            !enqueued ==> self@ == old(self)@,
            self.wf(),
    {
        self.try_enqueue(pid_val, tid_val, alarm_expired)
    }
