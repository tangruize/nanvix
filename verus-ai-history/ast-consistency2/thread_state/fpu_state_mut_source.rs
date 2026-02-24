    pub fn fpu_state_mut(&mut self) -> *mut FpuState {
        self.fpu_state.as_mut().get_mut() as *mut FpuState
    }
