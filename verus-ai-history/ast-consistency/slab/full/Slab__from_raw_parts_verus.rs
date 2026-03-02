    pub unsafe fn from_raw_parts(
        addr: usize,
        len: usize,
        block_size: usize,
    ) -> (result: Result<Slab, Error>)
        requires
            // Length must be valid and non-zero.
            len > 0,
            len < i32::MAX as usize,
            // Block size must be valid.
            block_size > 0,
            block_size < i32::MAX as usize,
            block_size <= len,
            // Block size must be a power of two.
            Self::spec_is_power_of_two(block_size as int),
            // Start address must be aligned to block size.
            addr % block_size == 0,
            addr > 0,
            // Memory region must not wrap around and fit in address space.
            (addr as int) + (len as int) <= (usize::MAX as int),
            // Total number of blocks must be a multiple of 8.
            (len / block_size) % (u8::BITS as usize) == 0,
            // Ensure we have enough blocks for a valid slab (at least 8).
            len / block_size >= 8,
            // Issue 1 FIX: Zero-initialization of the bitmap backing storage.
            // `raw_array_from_addr` zeroes the region before returning and its
            // postcondition exposes `is_zero` for every byte, which we rely on when
            // constructing the bitmap. No caller-side zeroing precondition is required.
        ensures
            // If result is Ok, these properties hold.
            result is Ok ==> {
                let slab = result->Ok_0;
                &&& slab.inv()
                &&& slab@.block_size == block_size as int
                // Freshly initialized: no blocks allocated (Set-based, no forall).
                &&& slab@.allocated_blocks =~= Set::<int>::empty()
                // The data address is at an offset from addr.
                &&& slab@.data_addr > addr as int
                &&& slab@.data_addr % (block_size as int) == 0
                // Number of data blocks is positive.
                &&& slab@.num_data_blocks > 0
                // Issue 6 FIX: Buffer bounds are recorded.
                &&& slab@.base_addr == addr as int
                &&& slab@.total_len == len as int
            },
    {
        // Check if length is invalid.
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid slab length"));
        }

        // Check if block size is valid.
        if block_size == 0 || block_size >= i32::MAX as usize || block_size > len {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid block size"));
        }

        // Check if the `block_size` is a power of two using the verified function.
        if !Self::is_power_of_two(block_size) {
            return Err(Error::new(ErrorCode::InvalidArgument, "block size is not a power of two"));
        }

        // At this point, is_power_of_two returned true, so spec_is_power_of_two holds.
        assert(Self::spec_is_power_of_two(block_size as int));

        // Check if `addr` is aligned to `block_size`.
        if addr % block_size != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned start address"));
        }

        // Compute layout of the slab allocator.
        let total_num_blocks: usize = len / block_size;
        if total_num_blocks % (u8::BITS as usize) != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid number of blocks"));
        }

        let index_len: usize = total_num_blocks / u8::BITS as usize;
        // Verus note: source uses `index_len.is_multiple_of(block_size)`.
        // `is_multiple_of()` is not available in Verus; `% == 0` is equivalent.
        let num_index_blocks: usize = (index_len / block_size)
            + if index_len % block_size == 0 { 0 } else { 1 };
        if num_index_blocks > total_num_blocks {
            return Err(Error::new(ErrorCode::InvalidArgument, "insufficient blocks for index"));
        }
        let num_data_blocks: usize = total_num_blocks - num_index_blocks;

        // Verus note: source uses `addr.add(num_index_blocks * block_size)` (pointer
        // arithmetic). Verus uses integer arithmetic; overflow safety proven via
        // preconditions. Prove num_index_blocks * block_size fits in usize.
        proof {
            // total_num_blocks >= 8 (from precondition), so index_len >= 1.
            assert(total_num_blocks >= 8usize);
            assert(index_len >= 1usize);
            // Prove num_index_blocks >= 1: either index_len / block_size >= 1,
            // or index_len < block_size so index_len % block_size > 0, adding 1.
            if index_len >= block_size {
                assert((index_len as int) / (block_size as int) >= 1) by(nonlinear_arith)
                    requires index_len >= block_size, block_size > 0int;
            } else {
                // index_len < block_size, so index_len / block_size == 0.
                // But index_len >= 1, so index_len % block_size == index_len > 0.
                assert((index_len as int) % (block_size as int) == (index_len as int)) by(nonlinear_arith)
                    requires 0 < index_len < block_size;
                assert(index_len % block_size != 0usize);
            }
            assert(num_index_blocks >= 1usize);
            // num_index_blocks <= total_num_blocks (from check above).
            // Since total_num_blocks >= 8 and num_index_blocks <= total_num_blocks / 8 + 1
            // (at most), we need to show num_index_blocks < total_num_blocks.
            // Actually: index_len = total_num_blocks / 8.
            // num_index_blocks = ceil(index_len / block_size) <= index_len (since block_size >= 1).
            // But index_len = total_num_blocks / 8, so num_index_blocks <= total_num_blocks / 8.
            // total_num_blocks / 8 < total_num_blocks (since total_num_blocks >= 8).
            // So num_index_blocks < total_num_blocks, hence num_data_blocks >= 1.
            assert(num_index_blocks <= total_num_blocks);
            // Prove num_data_blocks > 0: we need num_index_blocks < total_num_blocks.
            // index_len = total_num_blocks / 8 <= total_num_blocks / 8.
            // num_index_blocks <= (index_len / block_size) + 1.
            // block_size >= 1, so (index_len / block_size) <= index_len.
            // num_index_blocks <= index_len + 1 = total_num_blocks / 8 + 1.
            // For total_num_blocks >= 8: total_num_blocks / 8 + 1 <= total_num_blocks
            //   iff total_num_blocks / 8 <= total_num_blocks - 1
            //   iff total_num_blocks <= 8 * (total_num_blocks - 1) = 8*total_num_blocks - 8
            //   iff 8 <= 7 * total_num_blocks
            //   iff total_num_blocks >= 2 (true since >= 8).
            assert(num_index_blocks as int <= (index_len as int) + 1) by {
                assert((index_len as int) / (block_size as int) <= (index_len as int)) by(nonlinear_arith)
                    requires block_size >= 1int, index_len >= 0int;
            }
            assert((index_len as int) + 1 <= (total_num_blocks as int)) by {
                assert((total_num_blocks as int) / 8 + 1 <= (total_num_blocks as int)) by(nonlinear_arith)
                    requires total_num_blocks >= 8int;
            }
            assert(num_index_blocks < total_num_blocks);
            assert(num_data_blocks > 0usize);

            // Prove num_index_blocks * block_size <= len.
            Self::lemma_div_mul_le(len as int, block_size as int);
            Self::lemma_mul_inequality(num_index_blocks as int, total_num_blocks as int, block_size as int);
            // addr + num_index_blocks * block_size < addr + len <= usize::MAX.
            assert((num_index_blocks as int) * (block_size as int) < (total_num_blocks as int) * (block_size as int));
            assert((total_num_blocks as int) * (block_size as int) <= len as int);
            assert((addr as int) + (num_index_blocks as int) * (block_size as int) <= (addr as int) + (len as int));
            assert((addr as int) + (len as int) <= usize::MAX as int);
        }
        let data_addr: usize = addr + num_index_blocks * block_size;

        // Check if `data_addr` is aligned to `block_size`.
        if data_addr % block_size != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned data address"));
        }

        // Instantiate index.
        let storage: RawArray<u8> = raw_array_from_addr(addr, index_len)?;

        // Prove that all bytes in storage are zero (required by Bitmap::from_raw_array).
        proof {
            // raw_array_from_addr ensures is_zero for each element.
            // axiom_u8_zero_is_0 converts is_zero(t) to t == 0.
            assert forall|i: int| 0 <= i < storage@.len() implies storage@[i] == 0u8 by {
                axiom_u8_zero_is_0(storage@[i]);
            }
        }

        let mut index: Bitmap = Bitmap::from_raw_array(storage)?;

        // Prove key invariants before the loop.
        proof {
            // index@.number_of_bits() == index_len * 8 == total_num_blocks (since total_num_blocks % 8 == 0)
            assert(index.inv());
            assert(index@.number_of_bits() == index_len as int * 8);
            assert(total_num_blocks == index_len * 8);
            assert(index@.number_of_bits() == total_num_blocks as int);
            // num_index_blocks + num_data_blocks == total_num_blocks
            assert(num_index_blocks + num_data_blocks == total_num_blocks);
            // Therefore: num_index_blocks + num_data_blocks == index@.number_of_bits()
            assert(num_index_blocks as int + num_data_blocks as int == index@.number_of_bits());
            // num_index_blocks < total_num_blocks (from earlier check), so:
            assert(num_index_blocks < total_num_blocks);
        }

        // Initialize index: mark index blocks as allocated.
        let mut i: usize = 0;
        while i < num_index_blocks
            invariant
                index.inv(),
                i <= num_index_blocks,
                num_index_blocks < total_num_blocks,
                num_index_blocks > 0,
                num_data_blocks > 0,
                block_size > 0,
                data_addr > 0,
                num_index_blocks + num_data_blocks == total_num_blocks,
                index@.number_of_bits() == total_num_blocks as int,
                num_index_blocks as int + num_data_blocks as int == index@.number_of_bits(),
                // All bits from 0 to i are set (using set_bits directly).
                forall|j: int| #![trigger index@.set_bits.contains(j)]
                    0 <= j < i as int ==> index@.set_bits.contains(j),
                // All bits from i to end are not set (from initial state).
                forall|j: int| #![trigger index@.set_bits.contains(j)]
                    i as int <= j < index@.number_of_bits() ==> !index@.set_bits.contains(j),
            decreases num_index_blocks - i,
        {
            index.set(i)?;
            i = i + 1;
        }

        // After the loop, all index blocks are set.
        // Now prove the postconditions.
        let result_slab = Slab {
            index,
            data_addr,
            num_index_blocks,
            num_data_blocks,
            block_size,
            base_addr: addr,
            total_len: len,
        };

        proof {
            // Prove memory bounds conditions for the new invariant.
            // total_num_blocks = len / block_size.
            // num_data_blocks = total_num_blocks - num_index_blocks < total_num_blocks.
            // num_data_blocks * block_size < total_num_blocks * block_size = len.
            // Since len < i32::MAX < usize::MAX, we have num_data_blocks * block_size < usize::MAX.
            assert((num_data_blocks as int) < (total_num_blocks as int));
            // len = total_num_blocks * block_size (since len % block_size == 0 from the division).
            // Actually, len >= total_num_blocks * block_size but there might be remainder.
            // However, we know len / block_size = total_num_blocks, so:
            // total_num_blocks * block_size <= len < (total_num_blocks + 1) * block_size.
            // Use lemma to establish: (len / block_size) * block_size <= len.
            Self::lemma_div_mul_le(len as int, block_size as int);
            assert((total_num_blocks as int) == (len as int) / (block_size as int));
            assert((total_num_blocks as int) * (block_size as int) <= len as int);
            // num_data_blocks * block_size < total_num_blocks * block_size <= len < usize::MAX.
            Self::lemma_mul_inequality(num_data_blocks as int, total_num_blocks as int, block_size as int);
            assert((num_data_blocks as int) * (block_size as int) < (total_num_blocks as int) * (block_size as int));
            assert((len as int) < (usize::MAX as int));
            assert((num_data_blocks as int) * (block_size as int) <= (usize::MAX as int));

            // data_addr + num_data_blocks * block_size.
            // data_addr = addr + num_index_blocks * block_size.
            // data_addr + num_data_blocks * block_size = addr + num_index_blocks * block_size + num_data_blocks * block_size.
            //                                         = addr + (num_index_blocks + num_data_blocks) * block_size.
            //                                         = addr + total_num_blocks * block_size.
            //                                         <= addr + len (since total_num_blocks * block_size <= len).
            // From precondition: addr + len >= addr (no wrap), and len < i32::MAX.
            // So addr + len <= usize::MAX (implicitly, since addr + len doesn't wrap).
            assert((data_addr as int) == (addr as int) + (num_index_blocks as int) * (block_size as int));
            // Use distributive property.
            Self::lemma_distributive(num_index_blocks as int, num_data_blocks as int, block_size as int);
            assert((num_index_blocks as int) * (block_size as int) + (num_data_blocks as int) * (block_size as int)
                == (num_index_blocks as int + num_data_blocks as int) * (block_size as int));
            assert((data_addr as int) + (num_data_blocks as int) * (block_size as int)
                == (addr as int) + (num_index_blocks as int) * (block_size as int) + (num_data_blocks as int) * (block_size as int));
            assert((data_addr as int) + (num_data_blocks as int) * (block_size as int)
                == (addr as int) + (num_index_blocks as int + num_data_blocks as int) * (block_size as int));
            assert((num_index_blocks as int + num_data_blocks as int) == (total_num_blocks as int));
            assert((data_addr as int) + (num_data_blocks as int) * (block_size as int)
                == (addr as int) + (total_num_blocks as int) * (block_size as int));
            assert((total_num_blocks as int) * (block_size as int) <= len as int);
            assert((addr as int) + (total_num_blocks as int) * (block_size as int) <= (addr as int) + (len as int));
            // addr + len doesn't overflow (from precondition addr + len >= addr).
            // This means addr + len <= usize::MAX.
            assert((addr as int) + (len as int) <= (usize::MAX as int));
            assert((data_addr as int) + (num_data_blocks as int) * (block_size as int) <= (usize::MAX as int));

            // Issue 2 FIX: Prove metadata/data disjointness condition for invariant.
            // data_addr = addr + num_index_blocks * block_size.
            // Since addr > 0 (from precondition), we have:
            // data_addr = addr + num_index_blocks * block_size > num_index_blocks * block_size.
            // Therefore: data_addr >= num_index_blocks * block_size.
            assert((data_addr as int) == (addr as int) + (num_index_blocks as int) * (block_size as int));
            assert(addr > 0);
            assert((data_addr as int) > (num_index_blocks as int) * (block_size as int));
            assert((data_addr as int) >= (num_index_blocks as int) * (block_size as int));

            // Use the lemma to prove inv() holds.
            Self::lemma_inv_from_components(&result_slab);
            // Reveal view fields.
            Self::lemma_view_fields(&result_slab);
            // Prove that slab is empty (no data blocks allocated yet).
            Self::lemma_new_slab_is_empty(&result_slab);
            // Prove data_addr > addr (since data_addr = addr + index_region_size and index_region_size > 0).
            assert(result_slab.data_addr as int > addr as int);
            // Prove data_addr is aligned to block_size.
            assert(result_slab.data_addr as int % (block_size as int) == 0);
        }

        Ok(result_slab)
    }
