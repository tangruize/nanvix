    pub fn new(region: TruncatedMemoryRegion<PhysicalAddress>) -> Result<Self, Error> {
        Ok(Self {
            inner: Rc::new(RefCell::new(KpoolInner::new(region)?)),
        })
    }
