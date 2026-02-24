    pub fn has_room(&self, current_sp: usize, growth: usize) -> (result: bool)
        requires
            self.inv(),
            self@.contains_addr(current_sp as int) || current_sp as int == self.spec_top(),
        ensures
            result == (current_sp as int - growth as int >= self.spec_base()),
    {
        // Use subtraction instead of addition to avoid overflow.
        current_sp - self.base_addr >= growth
    }
