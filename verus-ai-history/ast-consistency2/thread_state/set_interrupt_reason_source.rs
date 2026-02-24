    pub(super) fn set_interrupt_reason(&mut self, reason: InterruptReason) {
        self.interrupt_reason = Some(reason);
    }
