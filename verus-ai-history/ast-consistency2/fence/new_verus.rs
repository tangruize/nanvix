    pub fn new(total: usize) -> (result: Self)
        ensures
            result@ == FenceView::spec_new(total as nat),
            result@.count == 0,
            result@.total == total as nat,
            result.inv(),
            total == 0 ==> result@.is_satisfied(),
            total > 0 ==> result@.is_waiting(),
    {
        proof { reveal(Fence::inv); }
        Fence { count: 0, total }
    }
