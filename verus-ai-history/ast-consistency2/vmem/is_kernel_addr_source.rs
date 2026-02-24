    fn is_kernel_addr(virt_addr: VirtualAddress) -> bool {
        !Self::is_user_addr(virt_addr)
    }
