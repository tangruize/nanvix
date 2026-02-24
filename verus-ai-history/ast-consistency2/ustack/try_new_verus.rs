    pub fn try_new(base_addr: usize) -> (result: Result<Self, Error>)
        requires
            spec_is_page_aligned(base_addr as int),
            base_addr as int + (USER_STACK_SIZE as int) <= usize::MAX as int,
        ensures
            result.is_ok(),
            result.is_ok() ==> {
                let stack = result.unwrap();
                &&& stack.inv()
                &&& stack.spec_base() == base_addr as int
                &&& stack.spec_size() == USER_STACK_SIZE as int
                &&& stack.spec_top() == base_addr as int + (USER_STACK_SIZE as int)
            },
    {
        // Validate page alignment.
        if base_addr % PAGE_ALIGNMENT != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "base address not page-aligned"));
        }

        // Check for overflow.
        if base_addr > usize::MAX - USER_STACK_SIZE {
            return Err(Error::new(ErrorCode::OutOfMemory, "address overflow"));
        }

        // Prove that USER_STACK_SIZE is page-aligned (128 * 4096 = 524288).
        proof {
            assert(USER_STACK_SIZE as int == USER_STACK_PAGES as int * (PAGE_SIZE as int));
            assert(spec_is_size_aligned(USER_STACK_SIZE as int));
        }

        // Prove page contiguity (consecutive pages are adjacent).
        proof {
            assert forall|i: int|
                0 <= i < USER_STACK_PAGES as int - 1
            implies
                #[trigger] (base_addr as int + (i + 1) * (PAGE_SIZE as int)) ==
                base_addr as int + i * (PAGE_SIZE as int) + (PAGE_SIZE as int)
            by {
                // page_end(i) = base_addr + (i+1) * PAGE_SIZE
                // page_start(i+1) = base_addr + (i+1) * PAGE_SIZE
                // They are equal by definition.
            }
        }

        let stack = UserStack { base_addr };

        // Prove the invariant holds.
        proof {
            // Prove size alignment.
            assert(spec_is_size_aligned(stack@.size())) by {
                assert(stack@.size() == USER_STACK_SIZE as int);
            }

            // Prove top alignment.
            assert(spec_is_page_aligned(stack@.top())) by {
                assert(stack@.top() == base_addr as int + (USER_STACK_SIZE as int));
                // base_addr is page-aligned and USER_STACK_SIZE is a multiple of PAGE_SIZE.
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
