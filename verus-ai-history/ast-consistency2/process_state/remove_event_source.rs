    pub fn remove_event(&mut self, ev: &Event) {
        self.events.retain(|o| o.event() != ev)
    }
