    pub fn is_user_addr(virt_addr: VirtualAddress) -> bool {
        virt_addr >= config::memory_layout::USER_BASE && virt_addr < config::memory_layout::USER_END
    }
