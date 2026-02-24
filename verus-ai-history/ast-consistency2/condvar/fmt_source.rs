    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "Condvar {{ sleeping: {:?} }}", self.inner.sleeping.borrow())
    }
