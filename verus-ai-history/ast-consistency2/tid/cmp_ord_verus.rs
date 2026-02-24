    pub fn cmp_ord(&self, other: &ThreadIdentifier) -> (result: core::cmp::Ordering)
        requires
            self.inv(),
            other.inv(),
        ensures
            (self@.value < other@.value) ==> result == core::cmp::Ordering::Less,
            (self@.value > other@.value) ==> result == core::cmp::Ordering::Greater,
            (self@.value == other@.value) ==> result == core::cmp::Ordering::Equal,
    {
        if self.value < other.value {
            core::cmp::Ordering::Less
        } else if self.value > other.value {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }
