    fn interrupt_reason(&mut self) -> Option<InterruptReason> {
        self.interrupt_reason.take()
    }
