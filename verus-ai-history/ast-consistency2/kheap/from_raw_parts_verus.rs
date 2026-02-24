    pub unsafe fn from_raw_parts(addr: usize, size: usize) -> (result: Result<Kheap, Error>)
        requires
            addr > 0,
            addr % PAGE_SIZE as usize == 0,
            size >= MIN_HEAP_SIZE as usize,
            size % MIN_HEAP_SIZE as usize == 0,
            // Size is a multiple of 8 (NUM_OF_SLABS), so division is exact.
            size % NUM_OF_SLABS == 0,
            // Ensure slab_size fits in i32 for Slab construction.
            (size / NUM_OF_SLABS) < i32::MAX as usize,
            (addr as int) + (size as int) <= (usize::MAX as int),
            // Alignment preconditions for each slab.
            // Since addr is page-aligned and size is a multiple of MIN_HEAP_SIZE = 8 * 131072 = 1048576,
            // slab_size = size/8 is a multiple of 131072 = 32 * 4096.
            // Therefore all slab addresses are 4096-aligned, hence aligned to all smaller block sizes.
            // We require these explicitly to help the verifier.
            (size as int) % (8int * 4096int) == 0,  // slab_size is multiple of 4096.
        ensures
            result is Ok ==> {
                let heap = result->Ok_0;
                &&& heap.inv()
                &&& heap@.is_empty()
            },
    {
        // Compute slab size.
        let slab_size: usize = size / NUM_OF_SLABS;

        // Prove the relationship between size and slab_size.
        proof {
            assert((size % NUM_OF_SLABS) as int == 0);
            assert((size as int) == (slab_size as int) * (NUM_OF_SLABS as int));
        }

        // Validate slab size is sufficient.
        if slab_size < MIN_SLAB_SIZE {
            return Err(Error::new(ErrorCode::InvalidArgument, "heap size too small"));
        }

        // Prove power-of-two properties for block sizes.
        proof {
            Slab::lemma_power_of_two_8();
            Slab::lemma_power_of_two_16();
            Slab::lemma_power_of_two_32();
            Slab::lemma_power_of_two_64();
            Slab::lemma_power_of_two_128();
            Slab::lemma_power_of_two_256();
            Slab::lemma_power_of_two_512();
            Slab::lemma_power_of_two_4096();

            // Prove alignment preconditions for all slabs.
            // addr is page-aligned (addr % PAGE_SIZE == 0, PAGE_SIZE = 4096).
            // slab_size = size / 8, and size % (8 * 4096) == 0, so slab_size % 4096 == 0.
            // Therefore addr + i * slab_size is always 4096-aligned, which implies alignment to all smaller powers of 2.

            // From preconditions:
            assert(addr % PAGE_SIZE == 0);
            assert(PAGE_SIZE == 4096);
            assert((addr as int) % 4096int == 0);

            // Prove slab_size % 4096 == 0 using the new precondition.
            Self::lemma_slab_size_alignment((size as int), (slab_size as int));
            assert((slab_size as int) % 4096int == 0);

            // Now prove (addr + i * slab_size) % block_size == 0 for each slab.
            // Since addr % 4096 == 0 and slab_size % 4096 == 0:
            // (addr + i * slab_size) % 4096 == 0 for all i.
            // And 4096 % block_size == 0 for all block_sizes.
            // So (addr + i * slab_size) % block_size == 0.

            // Use lemmas to prove alignment for each slab.
            // slab 0: offset=0, block_size=8.
            Self::lemma_mod_add_multiple((addr as int), (slab_size as int), 4096int, 0int);
            Self::lemma_mod_trans((addr as int) + 0int * (slab_size as int), 4096int, 8int);
            assert(((addr as int) + 0int * (slab_size as int)) % 8int == 0);

            // slab 1: offset=1, block_size=16.
            Self::lemma_mod_add_multiple((addr as int), (slab_size as int), 4096int, 1int);
            Self::lemma_mod_trans((addr as int) + 1int * (slab_size as int), 4096int, 16int);
            assert(((addr as int) + 1int * (slab_size as int)) % 16int == 0);

            // slab 2: offset=2, block_size=32.
            Self::lemma_mod_add_multiple((addr as int), (slab_size as int), 4096int, 2int);
            Self::lemma_mod_trans((addr as int) + 2int * (slab_size as int), 4096int, 32int);
            assert(((addr as int) + 2int * (slab_size as int)) % 32int == 0);

            // slab 3: offset=3, block_size=64.
            Self::lemma_mod_add_multiple((addr as int), (slab_size as int), 4096int, 3int);
            Self::lemma_mod_trans((addr as int) + 3int * (slab_size as int), 4096int, 64int);
            assert(((addr as int) + 3int * (slab_size as int)) % 64int == 0);

            // slab 4: offset=4, block_size=128.
            Self::lemma_mod_add_multiple((addr as int), (slab_size as int), 4096int, 4int);
            Self::lemma_mod_trans((addr as int) + 4int * (slab_size as int), 4096int, 128int);
            assert(((addr as int) + 4int * (slab_size as int)) % 128int == 0);

            // slab 5: offset=5, block_size=256.
            Self::lemma_mod_add_multiple((addr as int), (slab_size as int), 4096int, 5int);
            Self::lemma_mod_trans((addr as int) + 5int * (slab_size as int), 4096int, 256int);
            assert(((addr as int) + 5int * (slab_size as int)) % 256int == 0);

            // slab 6: offset=6, block_size=512.
            Self::lemma_mod_add_multiple((addr as int), (slab_size as int), 4096int, 6int);
            Self::lemma_mod_trans((addr as int) + 6int * (slab_size as int), 4096int, 512int);
            assert(((addr as int) + 6int * (slab_size as int)) % 512int == 0);

            // slab 7: offset=7, block_size=4096.
            Self::lemma_mod_add_multiple((addr as int), (slab_size as int), 4096int, 7int);
            Self::lemma_mod_trans((addr as int) + 7int * (slab_size as int), 4096int, 4096int);
            assert(((addr as int) + 7int * (slab_size as int)) % 4096int == 0);

            // Prove the new preconditions for from_raw_parts_at_offset.
            // slab_size >= MIN_SLAB_SIZE = 131072.
            assert(slab_size >= MIN_SLAB_SIZE);
            assert(MIN_SLAB_SIZE == 131072usize);

            // Prove slab_size % MIN_SLAB_SIZE == 0.
            // From precondition: size % MIN_HEAP_SIZE == 0, where MIN_HEAP_SIZE = 8 * MIN_SLAB_SIZE.
            // slab_size = size / 8, and size = m * MIN_HEAP_SIZE = m * 8 * MIN_SLAB_SIZE.
            // Therefore slab_size = m * MIN_SLAB_SIZE, so slab_size % MIN_SLAB_SIZE == 0.
            assert((slab_size as int) % (MIN_SLAB_SIZE as int) == 0) by {
                // size % MIN_HEAP_SIZE == 0, MIN_HEAP_SIZE = NUM_OF_SLABS * MIN_SLAB_SIZE = 8 * MIN_SLAB_SIZE.
                let m: int = (size as int) / (MIN_HEAP_SIZE as int);
                vstd::arithmetic::div_mod::lemma_fundamental_div_mod(size as int, MIN_HEAP_SIZE as int);
                assert((size as int) == m * (MIN_HEAP_SIZE as int));
                // MIN_HEAP_SIZE = 8 * MIN_SLAB_SIZE
                assert((MIN_HEAP_SIZE as int) == 8int * (MIN_SLAB_SIZE as int));
                assert((size as int) == m * 8 * (MIN_SLAB_SIZE as int));
                // slab_size = size / 8
                vstd::arithmetic::div_mod::lemma_div_multiples_vanish(m * (MIN_SLAB_SIZE as int), 8int);
                assert((slab_size as int) == m * (MIN_SLAB_SIZE as int));
                vstd::arithmetic::div_mod::lemma_mod_multiples_basic(m, MIN_SLAB_SIZE as int);
            }

            // Use lemma to prove block divisibility and count for each block size.
            Self::lemma_slab_block_divisibility((slab_size as int), 8int);
            Self::lemma_slab_block_divisibility((slab_size as int), 16int);
            Self::lemma_slab_block_divisibility((slab_size as int), 32int);
            Self::lemma_slab_block_divisibility((slab_size as int), 64int);
            Self::lemma_slab_block_divisibility((slab_size as int), 128int);
            Self::lemma_slab_block_divisibility((slab_size as int), 256int);
            Self::lemma_slab_block_divisibility((slab_size as int), 512int);
            Self::lemma_slab_block_divisibility((slab_size as int), 4096int);

            // Now the following assertions should hold.
            assert(slab_size / 4096 >= 8);
            assert(slab_size / 512 >= 8);
            assert(slab_size / 256 >= 8);
            assert(slab_size / 128 >= 8);
            assert(slab_size / 64 >= 8);
            assert(slab_size / 32 >= 8);
            assert(slab_size / 16 >= 8);
            assert(slab_size / 8 >= 8);

            assert((slab_size / 4096) % 8 == 0);
            assert((slab_size / 512) % 8 == 0);
            assert((slab_size / 256) % 8 == 0);
            assert((slab_size / 128) % 8 == 0);
            assert((slab_size / 64) % 8 == 0);
            assert((slab_size / 32) % 8 == 0);
            assert((slab_size / 16) % 8 == 0);
            assert((slab_size / 8) % 8 == 0);

            // Overflow preconditions.
            // offset < 8, so offset * slab_size < 8 * slab_size = size.
            // size <= usize::MAX (from precondition), so offset * slab_size < usize::MAX.
            // Similarly, addr + offset * slab_size < addr + size <= usize::MAX.
            assert(0int * (slab_size as int) <= (usize::MAX as int));
            assert(1int * (slab_size as int) <= (usize::MAX as int));
            assert(2int * (slab_size as int) <= (usize::MAX as int));
            assert(3int * (slab_size as int) <= (usize::MAX as int));
            assert(4int * (slab_size as int) <= (usize::MAX as int));
            assert(5int * (slab_size as int) <= (usize::MAX as int));
            assert(6int * (slab_size as int) <= (usize::MAX as int));
            assert(7int * (slab_size as int) <= (usize::MAX as int));

            assert((addr as int) + 0int * (slab_size as int) <= (usize::MAX as int));
            assert((addr as int) + 1int * (slab_size as int) <= (usize::MAX as int));
            assert((addr as int) + 2int * (slab_size as int) <= (usize::MAX as int));
            assert((addr as int) + 3int * (slab_size as int) <= (usize::MAX as int));
            assert((addr as int) + 4int * (slab_size as int) <= (usize::MAX as int));
            assert((addr as int) + 5int * (slab_size as int) <= (usize::MAX as int));
            assert((addr as int) + 6int * (slab_size as int) <= (usize::MAX as int));
            assert((addr as int) + 7int * (slab_size as int) <= (usize::MAX as int));
        }

        // Create the 8 slabs at consecutive memory regions.
        // Each slab starts at addr + i * slab_size.
        let slab_8: Slab = Slab::from_raw_parts_at_offset(addr, slab_size, 0, 8)?;
        let slab_16: Slab = Slab::from_raw_parts_at_offset(addr, slab_size, 1, 16)?;
        let slab_32: Slab = Slab::from_raw_parts_at_offset(addr, slab_size, 2, 32)?;
        let slab_64: Slab = Slab::from_raw_parts_at_offset(addr, slab_size, 3, 64)?;
        let slab_128: Slab = Slab::from_raw_parts_at_offset(addr, slab_size, 4, 128)?;
        let slab_256: Slab = Slab::from_raw_parts_at_offset(addr, slab_size, 5, 256)?;
        let slab_512: Slab = Slab::from_raw_parts_at_offset(addr, slab_size, 6, 512)?;
        let slab_4096: Slab = Slab::from_raw_parts_at_offset(addr, slab_size, 7, 4096)?;

        let heap: Kheap = Kheap {
            slab_8_bytes: slab_8,
            slab_16_bytes: slab_16,
            slab_32_bytes: slab_32,
            slab_64_bytes: slab_64,
            slab_128_bytes: slab_128,
            slab_256_bytes: slab_256,
            slab_512_bytes: slab_512,
            slab_4096_bytes: slab_4096,
            base_addr: Ghost(addr as int),
            total_size: Ghost(size as int),
        };

        proof {
            // Assert each slab has correct invariants.
            assert(heap.slab_8_bytes.inv());
            assert(heap.slab_16_bytes.inv());
            assert(heap.slab_32_bytes.inv());
            assert(heap.slab_64_bytes.inv());
            assert(heap.slab_128_bytes.inv());
            assert(heap.slab_256_bytes.inv());
            assert(heap.slab_512_bytes.inv());
            assert(heap.slab_4096_bytes.inv());

            // Block sizes from construction postconditions.
            assert(heap.slab_8_bytes@.block_size == 8);
            assert(heap.slab_16_bytes@.block_size == 16);
            assert(heap.slab_32_bytes@.block_size == 32);
            assert(heap.slab_64_bytes@.block_size == 64);
            assert(heap.slab_128_bytes@.block_size == 128);
            assert(heap.slab_256_bytes@.block_size == 256);
            assert(heap.slab_512_bytes@.block_size == 512);
            assert(heap.slab_4096_bytes@.block_size == 4096);

            // Key facts from construction for disjointness.
            let base: int = addr as int;
            let sz: int = slab_size as int;

            // Assert the ranges from postconditions.
            assert(heap.slab_8_bytes@.data_addr >= base + 0 * sz);
            assert(heap.slab_8_bytes@.data_addr + heap.slab_8_bytes@.num_data_blocks * heap.slab_8_bytes@.block_size <= base + 1 * sz);
            assert(heap.slab_16_bytes@.data_addr >= base + 1 * sz);
            assert(heap.slab_16_bytes@.data_addr + heap.slab_16_bytes@.num_data_blocks * heap.slab_16_bytes@.block_size <= base + 2 * sz);
            assert(heap.slab_32_bytes@.data_addr >= base + 2 * sz);
            assert(heap.slab_32_bytes@.data_addr + heap.slab_32_bytes@.num_data_blocks * heap.slab_32_bytes@.block_size <= base + 3 * sz);
            assert(heap.slab_64_bytes@.data_addr >= base + 3 * sz);
            assert(heap.slab_64_bytes@.data_addr + heap.slab_64_bytes@.num_data_blocks * heap.slab_64_bytes@.block_size <= base + 4 * sz);
            assert(heap.slab_128_bytes@.data_addr >= base + 4 * sz);
            assert(heap.slab_128_bytes@.data_addr + heap.slab_128_bytes@.num_data_blocks * heap.slab_128_bytes@.block_size <= base + 5 * sz);
            assert(heap.slab_256_bytes@.data_addr >= base + 5 * sz);
            assert(heap.slab_256_bytes@.data_addr + heap.slab_256_bytes@.num_data_blocks * heap.slab_256_bytes@.block_size <= base + 6 * sz);
            assert(heap.slab_512_bytes@.data_addr >= base + 6 * sz);
            assert(heap.slab_512_bytes@.data_addr + heap.slab_512_bytes@.num_data_blocks * heap.slab_512_bytes@.block_size <= base + 7 * sz);
            assert(heap.slab_4096_bytes@.data_addr >= base + 7 * sz);
            assert(heap.slab_4096_bytes@.data_addr + heap.slab_4096_bytes@.num_data_blocks * heap.slab_4096_bytes@.block_size <= base + 8 * sz);

            // Prove slabs_ordered (7 adjacency checks instead of 28 pairwise).
            assert(heap@.slab_precedes(&heap@.slab_8, &heap@.slab_16));
            assert(heap@.slab_precedes(&heap@.slab_16, &heap@.slab_32));
            assert(heap@.slab_precedes(&heap@.slab_32, &heap@.slab_64));
            assert(heap@.slab_precedes(&heap@.slab_64, &heap@.slab_128));
            assert(heap@.slab_precedes(&heap@.slab_128, &heap@.slab_256));
            assert(heap@.slab_precedes(&heap@.slab_256, &heap@.slab_512));
            assert(heap@.slab_precedes(&heap@.slab_512, &heap@.slab_4096));
            assert(heap@.slabs_ordered());
            assert(heap@.all_slabs_disjoint());

            // Prove all_slabs_within_extent.
            assert(heap@.base_addr == addr as int);
            assert(heap@.total_size == size as int);
            assert(heap@.all_slabs_within_extent());

            // Prove all_slabs_aligned.
            assert(heap@.slab_8.is_aligned());
            assert(heap@.slab_16.is_aligned());
            assert(heap@.slab_32.is_aligned());
            assert(heap@.slab_64.is_aligned());
            assert(heap@.slab_128.is_aligned());
            assert(heap@.slab_256.is_aligned());
            assert(heap@.slab_512.is_aligned());
            assert(heap@.slab_4096.is_aligned());
            assert(heap@.all_slabs_aligned());

            // Base addr and total size are valid.
            assert(heap.base_addr@ > 0);
            assert(heap.total_size@ > 0);

            // Prove inv.
            assert(heap.inv());

            // Prove is_empty by using the lemma.
            // from_raw_parts_at_offset gives us forall|i| !is_allocated(i).
            Slab::lemma_no_allocated_implies_empty(&heap.slab_8_bytes);
            Slab::lemma_no_allocated_implies_empty(&heap.slab_16_bytes);
            Slab::lemma_no_allocated_implies_empty(&heap.slab_32_bytes);
            Slab::lemma_no_allocated_implies_empty(&heap.slab_64_bytes);
            Slab::lemma_no_allocated_implies_empty(&heap.slab_128_bytes);
            Slab::lemma_no_allocated_implies_empty(&heap.slab_256_bytes);
            Slab::lemma_no_allocated_implies_empty(&heap.slab_512_bytes);
            Slab::lemma_no_allocated_implies_empty(&heap.slab_4096_bytes);

            assert(heap.slab_8_bytes@.is_empty());
            assert(heap.slab_16_bytes@.is_empty());
            assert(heap.slab_32_bytes@.is_empty());
            assert(heap.slab_64_bytes@.is_empty());
            assert(heap.slab_128_bytes@.is_empty());
            assert(heap.slab_256_bytes@.is_empty());
            assert(heap.slab_512_bytes@.is_empty());
            assert(heap.slab_4096_bytes@.is_empty());

            assert(heap@.slab_8.used() == 0);
            assert(heap@.slab_16.used() == 0);
            assert(heap@.slab_32.used() == 0);
            assert(heap@.slab_64.used() == 0);
            assert(heap@.slab_128.used() == 0);
            assert(heap@.slab_256.used() == 0);
            assert(heap@.slab_512.used() == 0);
            assert(heap@.slab_4096.used() == 0);

            assert(heap@.total_allocated() == 0);
            assert(heap@.is_empty());
        }

        Ok(heap)
    }
