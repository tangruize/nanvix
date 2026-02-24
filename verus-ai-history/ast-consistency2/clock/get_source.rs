    fn get(&self) -> (u32, u32) {
        (self.major.load(ORDER), self.minor.load(ORDER))
    }
