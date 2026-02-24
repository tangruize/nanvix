    pub fn new(frame_allocator: FrameAllocator) -> Self {
        Self {
            inner: Rc::new(RefCell::new(UpoolInner::new(frame_allocator))),
        }
    }
