    pub(super) fn take_kernel_stack(&mut self) -> Option<KernelStack> {
        self.kernel_stack.take()
    }
