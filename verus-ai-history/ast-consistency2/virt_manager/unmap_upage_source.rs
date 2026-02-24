    pub fn unmap_upage(
        &mut self,
        vmem: &mut Vmem,
        vaddr: PageAligned<VirtualAddress>,
    ) -> Result<(), Error> {
        let uframe: UserFrame = vmem.unmap(vaddr)?;
        self.physman.borrow_mut().free_user_frame(uframe)
    }
