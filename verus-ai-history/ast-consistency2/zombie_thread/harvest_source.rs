    pub fn harvest(mut self) -> (Option<KernelStack>, Option<UserStack>) {
        (self.state.take_kernel_stack(), self.state.take_user_stack())
    }
