    pub fn post_message(&mut self, message: Message) {
        self.mailbox.send(message)
    }
