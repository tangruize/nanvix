    pub fn new() -> Self {
        Self {
            inner: Arc::new(CondvarInner {
                sleeping: RefCell::new(LinkedList::new()),
            }),
        }
    }
