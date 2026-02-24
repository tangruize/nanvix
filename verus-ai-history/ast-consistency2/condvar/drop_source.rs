    fn drop(&mut self) {
        if !self.sleeping.borrow().is_empty() {
            panic!("{self:?}");
        }
    }
