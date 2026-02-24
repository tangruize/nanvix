    fn drop(&mut self) {
        while let Some((_pgtable_vaddr, user_page_table)) = self.user_page_tables.pop_front() {
            drop(user_page_table);
        }

        // Unmap all kernel private kernel pages.
        while let Some(kpage) = self.private_kernel_pages.pop_front() {
            drop(kpage)
        }

        // Unmap shared kernel pages.
        while let Some(entry) = self.kernel_pages.pop_front() {
            drop(entry);
        }

        // Unmap shared kernel page tables.
        while let Some(entry) = self.kernel_page_tables.pop_front() {
            drop(entry)
        }
    }
