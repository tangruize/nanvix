    pub(super) fn context_mut(&mut self) -> *mut ContextInformation {
        self.context.as_mut().get_mut() as *mut ContextInformation
    }
