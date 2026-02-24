    pub fn new(base_addr: usize, num_pages: usize) -> (result: Result<Self, Error>)
        requires
            spec_is_page_aligned(base_addr as int),
            0 < num_pages <= MAX_STACK_PAGES,
            base_addr as int + spec_pages_to_bytes(num_pages as int) <= usize::MAX as int,
        ensures
            // Liveness: When preconditions are met, allocation always succeeds.
            result.is_ok(),
            result.is_ok() ==> {
                let stack = result.unwrap();
                &&& stack.inv()
                &&& stack.spec_base() == base_addr as int
                &&& stack.spec_num_pages() == num_pages as int
                &&& stack.spec_size() == spec_pages_to_bytes(num_pages as int)
                &&& stack.spec_top() == base_addr as int + stack.spec_size()
            },
    {
        // Validate page alignment.
        if base_addr % PAGE_ALIGNMENT != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "base address not page-aligned"));
        }

        // Validate page count.
        if num_pages == 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "num_pages must be positive"));
        }

        if num_pages > MAX_STACK_PAGES {
            return Err(Error::new(ErrorCode::InvalidArgument, "num_pages exceeds maximum"));
        }

        // Check for overflow.
        let size: usize = num_pages * PAGE_SIZE;
        if base_addr > usize::MAX - size {
            return Err(Error::new(ErrorCode::OutOfMemory, "address overflow"));
        }

        let stack = KernelStack { base_addr, num_pages };

        // Prove the invariant holds.
        proof {
            // Prove size alignment.
            assert(spec_is_size_aligned(stack@.size())) by {
                assert(stack@.size() == num_pages as int * (PAGE_SIZE as int));
            }

            // Prove top alignment.
            assert(spec_is_page_aligned(stack@.top())) by {
                assert(stack@.top() == base_addr as int + num_pages as int * (PAGE_SIZE as int));
                // base_addr is page-aligned and size is a multiple of PAGE_SIZE.
                // So their sum is also page-aligned.
            }

            // Prove top > base.
            assert(stack@.top_greater_than_base()) by {
                assert(stack@.size() > 0);
            }

            // Prove pages are contiguous.
            assert(stack@.pages_are_contiguous());
        }

        Ok(stack)
    }
