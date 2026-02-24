    pub fn harvest_zombies(
        &mut self,
        mm: &mut VirtMemoryManager,
    ) -> Result<Option<(ProcessIdentifier, ExitStatus)>, Error> {
        let (mut zombie_threads, mut state, status): (
            VecDeque<ZombieThread>,
            Box<ProcessState>,
            ExitStatus,
        ) = match self.try_borrow_mut()?.harvest_zombies() {
            Some((zombie_threads, state, status)) => (zombie_threads, state, status),
            None => return Ok(None),
        };

        // Traverse the list of zombie threads.
        while let Some(zombie_thread) = zombie_threads.pop_front() {
            // Harvest zombie thread.
            if let (Some(_kernel_stack), Some(user_stack)) = zombie_thread.harvest() {
                // Traverse pages belonging to user stack.
                let base: usize = user_stack.base().into_raw_value();
                let top: usize = user_stack.top().into_raw_value();
                // TODO: Use an iterator for this.
                for raw_addr in (base..top).step_by(PAGE_SIZE) {
                    let vaddr: PageAligned<VirtualAddress> =
                        match PageAligned::from_raw_value(raw_addr) {
                            Ok(vaddr) => vaddr,
                            Err(_) => {
                                // SAFETY: the following condition is unreachable, because
                                // pages in the user stack are always page-aligned.
                                unreachable!("address conversion should succeed")
                            },
                        };
                    // Attempt to unmap page
                    if let Err(error) = mm.unmap_upage(state.vmem_mut(), vaddr) {
                        // We failed, but this is not too bad, as we will free all pages
                        // when wiping out the address space anyways.
                        warn!("failed to unmap page (vaddr={:?}, error={:?})", vaddr, error);
                    }
                }

                // Frames allocated to the user stack are freed when we exit this scope.
                // Frames allocated to the kernel stack are freed when we exit this scope.
            }
        }

        Ok(Some((state.pid(), status)))
    }
