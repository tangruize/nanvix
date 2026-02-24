    pub fn has_room(&self, current_sp: usize, growth: usize) -> (result: bool)
        requires
            self.inv(),
            // Note: current_sp == top() is valid because top() is the initial stack pointer
            // position before any pushes. The stack grows downward from top() toward base().
            self@.contains_addr(current_sp as int) || current_sp as int == self.spec_top(),
        ensures
            result == (current_sp as int - growth as int >= self.spec_base()),
    {
        // Use subtraction instead of addition to avoid overflow.
        // Safe: precondition guarantees current_sp >= base_addr, so no underflow.
        current_sp - self.base_addr >= growth
    }
