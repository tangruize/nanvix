    pub fn get_pte_index(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.pte_index(),
            result < PTES_PER_PGTAB,
    {
        // Use arithmetic equivalent of bit extraction: (addr / PAGE_SIZE) % 1024.
        let page_num: usize = self.raw_addr / PAGE_SIZE;
        page_num % PTES_PER_PGTAB
    }
