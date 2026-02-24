    pub fn deref_mut_len(&self) -> (result: usize)
        ensures
            result as int == spec_pgtab_entry_count(),
            result == INIT_PAGE_SIZE / 4,
    {
        INIT_PAGE_SIZE / 4
    }
