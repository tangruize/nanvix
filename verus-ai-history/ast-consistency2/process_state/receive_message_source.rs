    pub fn receive_message(&mut self, tid: ThreadIdentifier) -> Option<Message> {
        self.mailbox.receive(tid)
    }
