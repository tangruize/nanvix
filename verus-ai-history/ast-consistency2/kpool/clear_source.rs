    fn clear(&mut self) {
        for byte in self.iter_mut() {
            *byte = 0;
        }
    }
