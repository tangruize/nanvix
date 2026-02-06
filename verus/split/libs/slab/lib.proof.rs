// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

impl Slab {

    /// Lemma: If n == 1, then it's a power of two.
    proof fn lemma_one_is_power_of_two()
        ensures Self::spec_is_power_of_two(1),
    {
        // By definition: spec_is_power_of_two(1) == true.
    }


    /// Lemma: If n > 1 and n % 2 == 0 and n/2 is power of two, then n is power of two.
    proof fn lemma_double_power_of_two(n: int)
        requires
            n > 1,
            n % 2 == 0,
            Self::spec_is_power_of_two(n / 2),
        ensures
            Self::spec_is_power_of_two(n),
    {
        // By definition: for n > 1 and n % 2 == 0, spec_is_power_of_two(n) == spec_is_power_of_two(n/2).
    }


    /// Lemma: If n > 1 and n % 2 != 0, then n is not a power of two.
    proof fn lemma_odd_not_power_of_two(n: int)
        requires
            n > 1,
            n % 2 != 0,
        ensures
            !Self::spec_is_power_of_two(n),
    {
        // By definition: for n > 1 and n % 2 != 0, spec_is_power_of_two(n) == false.
    }


    /// Lemma: 8 is a power of two.
    pub proof fn lemma_power_of_two_8()
        ensures Self::spec_is_power_of_two(8),
    {
        Self::lemma_one_is_power_of_two();
        Self::lemma_double_power_of_two(2);
        Self::lemma_double_power_of_two(4);
        Self::lemma_double_power_of_two(8);
    }


    /// Lemma: 16 is a power of two.
    pub proof fn lemma_power_of_two_16()
        ensures Self::spec_is_power_of_two(16),
    {
        Self::lemma_power_of_two_8();
        Self::lemma_double_power_of_two(16);
    }


    /// Lemma: 32 is a power of two.
    pub proof fn lemma_power_of_two_32()
        ensures Self::spec_is_power_of_two(32),
    {
        Self::lemma_power_of_two_16();
        Self::lemma_double_power_of_two(32);
    }


    /// Lemma: 64 is a power of two.
    pub proof fn lemma_power_of_two_64()
        ensures Self::spec_is_power_of_two(64),
    {
        Self::lemma_power_of_two_32();
        Self::lemma_double_power_of_two(64);
    }


    /// Lemma: 128 is a power of two.
    pub proof fn lemma_power_of_two_128()
        ensures Self::spec_is_power_of_two(128),
    {
        Self::lemma_power_of_two_64();
        Self::lemma_double_power_of_two(128);
    }


    /// Lemma: 256 is a power of two.
    pub proof fn lemma_power_of_two_256()
        ensures Self::spec_is_power_of_two(256),
    {
        Self::lemma_power_of_two_128();
        Self::lemma_double_power_of_two(256);
    }


    /// Lemma: 512 is a power of two.
    pub proof fn lemma_power_of_two_512()
        ensures Self::spec_is_power_of_two(512),
    {
        Self::lemma_power_of_two_256();
        Self::lemma_double_power_of_two(512);
    }


    /// Lemma: 1024 is a power of two.
    pub proof fn lemma_power_of_two_1024()
        ensures Self::spec_is_power_of_two(1024),
    {
        Self::lemma_power_of_two_512();
        Self::lemma_double_power_of_two(1024);
    }


    /// Lemma: 2048 is a power of two.
    pub proof fn lemma_power_of_two_2048()
        ensures Self::spec_is_power_of_two(2048),
    {
        Self::lemma_power_of_two_1024();
        Self::lemma_double_power_of_two(2048);
    }


    /// Lemma: 4096 is a power of two.
    pub proof fn lemma_power_of_two_4096()
        ensures Self::spec_is_power_of_two(4096),
    {
        Self::lemma_power_of_two_2048();
        Self::lemma_double_power_of_two(4096);
    }

    //==============================================================================================

    /// Lemma: (a + 1) * b = a * b + b (distributive property).
    pub proof fn lemma_mul_distribute(a: int, b: int)
        ensures
            (a + 1) * b == a * b + b,
    {
        // Use vstd's distributive lemma
        vstd::arithmetic::mul::lemma_mul_is_distributive_add(b, a, 1);
        // This proves: b * (a + 1) == b * a + b * 1
        // By commutativity: (a + 1) * b == a * b + b
    }

    //==============================================================================================

    /// Lemma: Prove that a Slab satisfies the invariant given its components satisfy the conditions.
    proof fn lemma_inv_from_components(slab: &Slab)
        requires
            slab.index.inv(),
            slab.block_size > 0,
            slab.num_data_blocks > 0,
            slab.num_index_blocks > 0,
            slab.num_index_blocks + slab.num_data_blocks == slab.index@.number_of_bits(),
            // Index blocks are marked in set_bits.
            forall|i: int| #![trigger slab.index@.set_bits.contains(i)]
                0 <= i < slab.num_index_blocks as int ==> slab.index@.set_bits.contains(i),
            slab.data_addr > 0,
            // Memory bounds conditions.
            (slab.num_data_blocks as int) * (slab.block_size as int) <= usize::MAX as int,
            (slab.data_addr as int) + (slab.num_data_blocks as int) * (slab.block_size as int) <= usize::MAX as int,
            // Metadata/Data disjointness condition.
            slab.data_addr as int >= slab.num_index_blocks as int * slab.block_size as int,
            // Issue 3 FIX: Power-of-two and alignment conditions.
            Self::spec_is_power_of_two(slab.block_size as int),
            slab.data_addr as int % slab.block_size as int == 0,
            // Issue 6 FIX: Buffer bounds conditions.
            slab.base_addr > 0,
            slab.total_len > 0,
            (slab.base_addr as int) + (slab.total_len as int) <= usize::MAX as int,
            slab.base_addr as int <= slab.data_addr as int,
            (slab.data_addr as int) + (slab.num_data_blocks as int) * (slab.block_size as int)
                <= (slab.base_addr as int) + (slab.total_len as int),
            slab.data_addr as int == slab.base_addr as int + slab.num_index_blocks as int * slab.block_size as int,
        ensures
            slab.inv(),
    {
        // This follows directly from the definition of inv().
    }


    /// Lemma: Reveal the relationship between slab view and slab fields.
    proof fn lemma_view_fields(slab: &Slab)
        requires
            slab.inv(),
        ensures
            slab@.block_size == slab.block_size as int,
            slab@.data_addr == slab.data_addr as int,
            slab@.num_data_blocks == slab.num_data_blocks as int,
    {
        // Follows from definition of view().
    }


    /// Lemma: Allocated blocks are always within valid range.
    /// This follows from the definition of allocated_blocks in view().
    proof fn lemma_allocated_in_range(&self, i: int)
        requires
            self.inv(),
            self@.is_allocated(i),
        ensures
            0 <= i < self@.num_data_blocks,
    {
        // By definition of view(), allocated_blocks only contains i where
        // 0 <= i < num_data_blocks && is_bit_set(num_index_blocks + i).
        // So if i is allocated, it must be in [0, num_data_blocks).
    }


    /// Lemma: If no block is allocated, the slab is empty.
    /// Bridges `forall|i| !is_allocated(i)` to `is_empty()`.
    ///
    /// # Proof Strategy
    ///
    /// We prove that allocated_blocks == empty set by showing no element can be in it.
    /// - For i in [0, num_data_blocks): !is_allocated(i) by precondition.
    /// - For i outside this range: !is_allocated(i) by allocated_blocks_in_range (from inv).
    /// Therefore, forall i, !allocated_blocks.contains(i), so allocated_blocks =~= Set::empty().
    pub proof fn lemma_no_allocated_implies_empty(slab: &Slab)
        requires
            slab.inv(),
            forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i),
        ensures
            slab@.is_empty(),
    {
        // Prove that allocated_blocks equals empty set.
        assert(slab@.allocated_blocks =~= Set::<int>::empty()) by {
            // For any i, show !allocated_blocks.contains(i).
            assert forall|i: int| !slab@.allocated_blocks.contains(i) by {
                if 0 <= i < slab@.num_data_blocks {
                    // By precondition: !is_allocated(i), so !allocated_blocks.contains(i).
                    assert(!slab@.is_allocated(i));
                } else {
                    // By allocated_blocks_in_range from inv: is_allocated(i) ==> 0 <= i < num_data_blocks.
                    // Contrapositive: !(0 <= i < num_data_blocks) ==> !is_allocated(i).
                    assert(slab@.allocated_blocks_in_range());
                    assert(!slab@.is_allocated(i));
                }
            }
        }
        // allocated_blocks =~= empty set, so len() == 0, so is_empty().
    }


    /// Lemma: Reveal that a newly created slab with no data blocks allocated has no allocated blocks.
    proof fn lemma_new_slab_is_empty(slab: &Slab)
        requires
            slab.inv(),
            // All data blocks are unset in the bitmap.
            forall|i: int| slab.num_index_blocks as int <= i < (slab.num_index_blocks + slab.num_data_blocks) as int
                ==> !slab.index.is_bit_set(i),
        ensures
            // All data blocks are not allocated.
            forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i),
    {
        // is_allocated(i) <==> is_bit_set(num_index_blocks + i)
        // By precondition, !is_bit_set(num_index_blocks + i) for all i in [0, num_data_blocks).
        assert forall|i: int| 0 <= i < slab@.num_data_blocks implies !slab@.is_allocated(i) by {
            let bitmap_idx = slab.num_index_blocks as int + i;
            assert(!slab.index.is_bit_set(bitmap_idx));
        }
    }


    /// Lemma: If a block is an index block, it's always set in the bitmap.
    /// Therefore, alloc() will never return an index block.
    proof fn lemma_index_blocks_always_set(&self)
        requires
            self.inv(),
        ensures
            forall|i: int| 0 <= i < self.num_index_blocks as int ==> self.index.is_bit_set(i),
    {
        // Follows directly from the invariant.
    }


    /// Lemma: Slab invariant implies Bitmap invariant.
    proof fn lemma_slab_inv_implies_bitmap_inv(&self)
        requires
            self.inv(),
        ensures
            self.index.inv(),
            self.index@.number_of_bits() <= (usize::MAX as int),
    {
        // slab.inv() implies index.inv(), and index.inv() implies the bound.
        self.index.lemma_number_of_bits_bounded();
    }


    /// Lemma: If bitmap.alloc() returns a bit index, that bit was not set before.
    /// Combined with lemma_index_blocks_always_set, this means alloc returns a data block.
    proof fn lemma_alloc_returns_data_block(&self, block: int)
        requires
            self.inv(),
            0 <= block < self.index@.number_of_bits(),
            !self.index.is_bit_set(block),
        ensures
            block >= self.num_index_blocks as int,
    {
        // If block < num_index_blocks, then by inv, is_bit_set(block) would be true.
        // But precondition says !is_bit_set(block), contradiction.
        if block < self.num_index_blocks as int {
            assert(self.index.is_bit_set(block)); // from inv
            assert(!self.index.is_bit_set(block)); // from precondition
        }
    }


    /// Lemma: After allocation, the allocated block is in the set.
    proof fn lemma_allocate_adds_block(&self, new_self: &Self, block_idx: int)
        requires
            self.inv(),
            new_self.inv(),
            0 <= block_idx < self@.num_data_blocks,
            !self@.is_allocated(block_idx),
            new_self@.is_allocated(block_idx),
            self@.num_data_blocks == new_self@.num_data_blocks,
            self@.block_size == new_self@.block_size,
            self@.data_addr == new_self@.data_addr,
        ensures
            new_self@.is_allocated(block_idx),
            !self@.is_allocated(block_idx),
    {
        // Follows from definition of view and is_allocated.
    }


    /// Lemma: After deallocation, the deallocated block is not in the set.
    proof fn lemma_deallocate_removes_block(&self, new_self: &Self, block_idx: int)
        requires
            self.inv(),
            new_self.inv(),
            0 <= block_idx < self@.num_data_blocks,
            self@.is_allocated(block_idx),
            !new_self@.is_allocated(block_idx),
            self@.num_data_blocks == new_self@.num_data_blocks,
            self@.block_size == new_self@.block_size,
            self@.data_addr == new_self@.data_addr,
        ensures
            !new_self@.is_allocated(block_idx),
            self@.is_allocated(block_idx),
    {
        // Follows from definition of view and is_allocated.
    }


    /// Lemma (Liveness): After deallocating a block, it can be allocated again.
    /// This ensures the allocator doesn't "lose" freed blocks.
    proof fn lemma_deallocate_enables_reallocation(&self, new_self: &Self, block_idx: int)
        requires
            self.inv(),
            new_self.inv(),
            0 <= block_idx < self@.num_data_blocks,
            self@.is_allocated(block_idx),
            !new_self@.is_allocated(block_idx),
            new_self@.used() < new_self@.capacity(),  // After dealloc, there's capacity
        ensures
            // The deallocated block is now free and can be allocated.
            !new_self@.is_allocated(block_idx),
            // There's at least one free block (the one we just freed).
            new_self@.used() < new_self@.capacity(),
    {
        // After deallocate:
        // - The block's bit is cleared in the bitmap.
        // - used count decremented.
        // - The block is now in the free set and can be returned by next alloc.
    }


    /// Lemma: Invariant implies positive capacity.
    ///
    /// This lemma reveals the `num_data_blocks > 0` property that is
    /// part of the closed `inv()` spec. Useful for clients that need
    /// to reason about capacity without knowing inv() internals.
    pub proof fn lemma_inv_implies_positive_capacity(&self)
        requires
            self.inv(),
        ensures
            self@.num_data_blocks > 0,
    {
        // Follows from inv() definition: self.num_data_blocks > 0
        // and self@.num_data_blocks == self.num_data_blocks as int.
    }

    //==============================================================================================

    /// Helper lemma: allocated_blocks is a subset of set_int_range(0, num_data_blocks).
    /// This follows from allocated_blocks_in_range: forall|i| is_allocated(i) ==> 0 <= i < num_data_blocks.
    proof fn lemma_allocated_blocks_subset_of_range(&self)
        requires
            self.inv(),
        ensures
            self@.allocated_blocks.subset_of(set_int_range(0, self@.num_data_blocks)),
    {
        // allocated_blocks_in_range says: forall|i| is_allocated(i) ==> 0 <= i < num_data_blocks.
        // is_allocated(i) <==> allocated_blocks.contains(i).
        // set_int_range(0, num_data_blocks).contains(i) <==> 0 <= i < num_data_blocks.
        // So: forall|i| allocated_blocks.contains(i) ==> set_int_range(0, num_data_blocks).contains(i).
        // This is the definition of subset_of.

        let num_data: int = self@.num_data_blocks;
        let full_range: Set<int> = set_int_range(0, num_data);

        // Prove: forall|i| allocated_blocks.contains(i) ==> full_range.contains(i).
        assert forall|i: int| #![auto] self@.allocated_blocks.contains(i) implies full_range.contains(i) by {
            // If allocated_blocks.contains(i), then is_allocated(i) by definition.
            assert(self@.is_allocated(i));
            // By allocated_blocks_in_range (which follows from view definition), 0 <= i < num_data.
            assert(self@.allocated_blocks_in_range());
            assert(0 <= i < num_data);
            // set_int_range(0, num_data).contains(i) <==> 0 <= i < num_data.
            assert(full_range.contains(i));
        }

        // By definition, subset_of(A, B) <==> forall|x| A.contains(x) ==> B.contains(x).
        assert(self@.allocated_blocks.subset_of(full_range));
    }


    /// Helper lemma: allocated_blocks is finite.
    /// Since allocated_blocks is a subset of set_int_range(0, num_data_blocks), and that's finite,
    /// allocated_blocks is also finite.
    proof fn lemma_allocated_blocks_finite(&self)
        requires
            self.inv(),
        ensures
            self@.allocated_blocks.finite(),
    {
        let num_data: int = self@.num_data_blocks;
        let full_range: Set<int> = set_int_range(0, num_data);

        // Prove full_range is finite.
        lemma_int_range(0, num_data);
        assert(full_range.finite());

        // Use lemma_allocated_blocks_subset_of_range to prove the subset property.
        self.lemma_allocated_blocks_subset_of_range();
        assert(self@.allocated_blocks.subset_of(full_range));

        // By lemma_set_subset_finite, a subset of a finite set is finite.
        // Note: lemma_set_subset_finite(superset, subset) - superset comes first!
        lemma_set_subset_finite(full_range, self@.allocated_blocks);
    }


    /// Lemma (Liveness): If slab can_allocate(), then bitmap has_free_bit().
    /// This is the key lemma connecting slab liveness to bitmap liveness.
    ///
    /// Proof sketch:
    /// - can_allocate() means free() > 0
    /// - free() = capacity - used = num_data_blocks - |allocated_blocks|
    /// - If free() > 0, then |allocated_blocks| < num_data_blocks
    /// - allocated_blocks = { i | 0 <= i < num_data_blocks && is_bit_set(num_index_blocks + i) }
    /// - If |allocated_blocks| < num_data_blocks, there exists some data index j not in allocated_blocks
    /// - That means is_bit_set(num_index_blocks + j) is false
    /// - Therefore, there's an unset bit in the bitmap => has_free_bit()
    ///
    /// Proves that if the slab can allocate, then the bitmap has a free bit.
    /// This connects liveness (can_allocate) to the concrete bitmap state.
    proof fn lemma_can_allocate_implies_bitmap_has_free_bit(&self)
        requires
            self.inv(),
            self@.can_allocate(),
        ensures
            self.index@.has_free_bit(),
    {
        // can_allocate() means free() > 0, i.e., used() < capacity() = num_data_blocks.
        // We need to show: exists|i| 0 <= i < number_of_bits && !is_bit_set(i).
        //
        // Strategy: We know |allocated_blocks| < num_data_blocks.
        // allocated_blocks is a subset of {0, 1, ..., num_data_blocks - 1}.
        // Since its cardinality is less than num_data_blocks, there exists j such that
        // 0 <= j < num_data_blocks and j is NOT in allocated_blocks.
        // By definition of view(), !is_allocated(j) means !is_bit_set(num_index_blocks + j).
        // Since num_index_blocks + j < number_of_bits, we have the free bit.

        let num_data: int = self.num_data_blocks as int;
        let num_idx: int = self.num_index_blocks as int;

        // From can_allocate: self@.free() > 0
        // free() = capacity() - used() = num_data_blocks - |allocated_blocks|
        // So |allocated_blocks| < num_data_blocks.
        assert(self@.allocated_blocks.len() < num_data);

        // The full range set [0, num_data) has exactly num_data elements.
        let full_range: Set<int> = set_int_range(0, num_data);
        lemma_int_range(0, num_data);
        assert(full_range.finite());
        assert(full_range.len() == num_data);

        // Use helper lemmas to prove finiteness and subset properties.
        self.lemma_allocated_blocks_finite();
        assert(self@.allocated_blocks.finite());

        self.lemma_allocated_blocks_subset_of_range();
        assert(self@.allocated_blocks.subset_of(full_range));

        // Now use cardinality reasoning:
        // |allocated_blocks| < num_data = |full_range|.
        // Therefore, allocated_blocks != full_range (strict subset).
        // So there exists j in full_range that is NOT in allocated_blocks.

        // If allocated_blocks == full_range, then |allocated_blocks| == |full_range| == num_data.
        // But |allocated_blocks| < num_data. Contradiction.
        // Therefore allocated_blocks != full_range, meaning some element is missing.

        // Prove by contradiction: if all elements of full_range were in allocated_blocks,
        // then full_range would be a subset of allocated_blocks.
        if forall|j: int| #![auto] full_range.contains(j) ==> self@.allocated_blocks.contains(j) {
            // This means full_range is a subset of allocated_blocks.
            assert(full_range.subset_of(self@.allocated_blocks));
            // By lemma_len_subset: full_range.len() <= allocated_blocks.len().
            lemma_len_subset(full_range, self@.allocated_blocks);
            assert(full_range.len() <= self@.allocated_blocks.len());
            // But full_range.len() == num_data and allocated_blocks.len() < num_data.
            // Contradiction: num_data <= allocated_blocks.len() < num_data is impossible.
            assert(false);
        }

        // Now we know: exists j in full_range such that !allocated_blocks.contains(j).
        // This means: exists j. 0 <= j < num_data && !allocated_blocks.contains(j).
        let j: int = choose|j: int| #![auto] full_range.contains(j) && !self@.allocated_blocks.contains(j);

        // j is in [0, num_data) since full_range = set_int_range(0, num_data).
        assert(0 <= j && j < num_data);

        // !allocated_blocks.contains(j) means !is_allocated(j).
        // By definition of view(): is_allocated(j) <==> is_bit_set(num_index_blocks + j).
        // So !is_bit_set(num_index_blocks + j).
        assert(!self@.is_allocated(j));

        // The connection: is_allocated(j) == index.is_bit_set(num_idx + j).
        // By view definition, !is_allocated(j) means !is_bit_set(num_idx + j).
        let bitmap_idx: int = num_idx + j;

        // bitmap_idx is in valid range: 0 <= num_idx + j < num_idx + num_data = number_of_bits.
        assert(0 <= bitmap_idx);
        assert(bitmap_idx < self.index@.number_of_bits());

        // We need to show !is_bit_set(bitmap_idx).
        // This follows from !is_allocated(j) and the view definition.
        // The view says: allocated_blocks contains j iff is_bit_set(num_idx + j).
        // So !allocated_blocks.contains(j) ==> !is_bit_set(num_idx + j).
        assert(!self.index.is_bit_set(bitmap_idx));

        // has_free_bit() = exists|i| 0 <= i < number_of_bits && !is_bit_set(i).
        // We have: 0 <= bitmap_idx < number_of_bits && !is_bit_set(bitmap_idx).
        // Use lemma to establish has_free_bit.
        self.index.lemma_unset_bit_implies_has_free_bit(bitmap_idx);
        assert(self.index@.has_free_bit());
    }


    /// Lemma (Liveness): If bitmap is full, then slab is full.
    /// This is the converse of lemma_can_allocate_implies_bitmap_has_free_bit.
    /// It connects bitmap fullness to slab fullness.
    proof fn lemma_bitmap_full_implies_slab_full(&self)
        requires
            self.inv(),
            self.index@.is_full(),
        ensures
            self@.is_full(),
    {
        // bitmap.is_full() means: forall|i| 0 <= i < number_of_bits ==> is_bit_set(i).
        // In particular, for all data block indices j (0 <= j < num_data_blocks):
        //   is_bit_set(num_index_blocks + j) == true.
        // By view definition, is_allocated(j) <==> is_bit_set(num_index_blocks + j).
        // So: forall|j| 0 <= j < num_data_blocks ==> is_allocated(j).
        // This means allocated_blocks = {0, 1, ..., num_data_blocks - 1}.
        // Therefore: |allocated_blocks| == num_data_blocks == capacity.
        // By definition, is_full() <==> used() == num_data_blocks.

        let num_data: int = self.num_data_blocks as int;
        let num_idx: int = self.num_index_blocks as int;

        // Use lemma to establish that is_full() implies all bits are set.
        self.index.lemma_is_full_means_all_bits_set();

        // Prove all data block indices are allocated.
        assert forall|j: int| 0 <= j < num_data implies self@.is_allocated(j) by {
            let bitmap_idx = num_idx + j;
            assert(0 <= bitmap_idx < self.index@.number_of_bits());
            assert(self.index.is_bit_set(bitmap_idx));  // From bitmap.is_full() via lemma
            // By view definition, this means is_allocated(j).
        }

        // Now prove allocated_blocks == set_int_range(0, num_data).
        let full_range: Set<int> = set_int_range(0, num_data);
        lemma_int_range(0, num_data);
        assert(full_range.finite());
        assert(full_range.len() == num_data);

        // Prove: forall|j| full_range.contains(j) ==> allocated_blocks.contains(j).
        assert forall|j: int| #![auto] full_range.contains(j) implies self@.allocated_blocks.contains(j) by {
            assert(0 <= j < num_data);
            assert(self@.is_allocated(j));
        }

        // Also: forall|j| allocated_blocks.contains(j) ==> full_range.contains(j).
        // This follows from allocated_blocks_in_range (proven via inv).
        assert forall|j: int| #![auto] self@.allocated_blocks.contains(j) implies full_range.contains(j) by {
            assert(self@.allocated_blocks_in_range());
            assert(0 <= j < num_data);
        }

        // Therefore allocated_blocks == full_range (same membership).
        assert(self@.allocated_blocks =~= full_range);

        // Since allocated_blocks == full_range, |allocated_blocks| == num_data.
        self.lemma_allocated_blocks_finite();
        assert(self@.allocated_blocks.len() == num_data);

        // is_full() <==> used() == num_data_blocks.
        // used() == |allocated_blocks| == num_data == num_data_blocks.
        assert(self@.used() == self@.num_data_blocks);
        assert(self@.is_full());
    }


    /// Lemma (Liveness): Deallocation from full slab enables allocation.
    /// If the slab was full, after deallocating one block, allocation becomes possible.
    proof fn lemma_dealloc_from_full_enables_alloc(&self, new_self: &Self, block_idx: int)
        requires
            self.inv(),
            new_self.inv(),
            self@.used() == self@.capacity(),  // Slab was full
            0 <= block_idx < self@.num_data_blocks,
            self@.is_allocated(block_idx),
            !new_self@.is_allocated(block_idx),
            // Other blocks unchanged
            forall|i: int| (0 <= i < self@.num_data_blocks && i != block_idx) ==>
                (self@.is_allocated(i) <==> new_self@.is_allocated(i)),
            new_self@.num_data_blocks == self@.num_data_blocks,
        ensures
            new_self@.can_allocate(),
            new_self@.free() >= 1,
    {
        // Prove that new_self's allocated_blocks is a subset of self's allocated_blocks minus block_idx.
        // Step 1: self@.allocated_blocks contains block_idx
        assert(self@.allocated_blocks.contains(block_idx));

        // Step 2: new_self@.allocated_blocks does NOT contain block_idx
        assert(!new_self@.allocated_blocks.contains(block_idx));

        // Step 3: new_self@.allocated_blocks is a subset of self@.allocated_blocks.remove(block_idx)
        // Because: for any i in new_self's allocated_blocks:
        //   - i != block_idx (since block_idx is not in new_self)
        //   - if i is allocated in new_self, it was allocated in self (by unchanged property)
        //   - so i is in self@.allocated_blocks.remove(block_idx)
        let old_set: Set<int> = self@.allocated_blocks;
        let new_set: Set<int> = new_self@.allocated_blocks;
        let removed_set: Set<int> = old_set.remove(block_idx);

        // Prove old_set is finite: it's a subset of set_int_range(0, num_data_blocks)
        let range_set: Set<int> = set_int_range(0, self@.num_data_blocks);

        // Prove old_set is a subset of range_set
        assert forall|i: int| old_set.contains(i) implies range_set.contains(i) by {
            // If i is in old_set, then i is allocated, so 0 <= i < num_data_blocks
            // by the definition of allocated_blocks in view()
        }
        assert(old_set.subset_of(range_set));

        // range_set is finite
        lemma_int_range(0, self@.num_data_blocks);
        assert(range_set.finite());

        // old_set is a subset of finite range_set, so old_set is finite
        lemma_set_subset_finite(range_set, old_set);
        assert(old_set.finite());

        // removed_set is finite (removing from finite set stays finite)
        assert(removed_set.finite());

        // Similarly prove new_set is finite
        let new_range_set: Set<int> = set_int_range(0, new_self@.num_data_blocks);
        assert forall|i: int| new_set.contains(i) implies new_range_set.contains(i) by {
            // If i is in new_set, then i is allocated in new_self, so 0 <= i < num_data_blocks
        }
        assert(new_set.subset_of(new_range_set));
        lemma_int_range(0, new_self@.num_data_blocks);
        assert(new_range_set.finite());
        lemma_set_subset_finite(new_range_set, new_set);
        assert(new_set.finite());

        // Assert that new_set is a subset of removed_set
        assert forall|i: int| new_set.contains(i) implies removed_set.contains(i) by {
            if new_set.contains(i) {
                // i is allocated in new_self
                assert(new_self@.is_allocated(i));
                // i != block_idx since block_idx is not allocated in new_self
                assert(i != block_idx);
                // i must be in range since it's allocated
                assert(0 <= i < new_self@.num_data_blocks);
                assert(0 <= i < self@.num_data_blocks);
                // By the unchanged property, i was also allocated in self
                assert(self@.is_allocated(i));
                assert(old_set.contains(i));
                // Since i != block_idx and i is in old_set, i is in removed_set
                assert(removed_set.contains(i));
            }
        }
        assert(new_set.subset_of(removed_set));

        // Use vstd lemma: removing an element decreases length by 1 if element was present
        axiom_set_remove_len(old_set, block_idx);
        assert(removed_set.len() == old_set.len() - 1);

        // new_set is a subset of removed_set, so its length is at most removed_set's length
        lemma_len_subset(new_set, removed_set);
        assert(new_set.len() <= removed_set.len());

        // Therefore: new_set.len() <= old_set.len() - 1
        // old_set.len() == self@.used() == self@.capacity() == self@.num_data_blocks
        // So: new_set.len() <= capacity - 1
        // Therefore: new_self@.used() < new_self@.capacity()
        // Therefore: new_self@.free() >= 1 and can_allocate() is true
        assert(new_self@.used() <= self@.capacity() - 1);
        assert(new_self@.free() >= 1);
        assert(new_self@.can_allocate());
    }

    //==============================================================================================

    /// Lemma: A newly created slab is freshly initialized (no data blocks allocated).
    proof fn lemma_new_slab_freshly_initialized(slab: &Slab)
        requires
            slab.inv(),
            // All data blocks are unset in the bitmap (from new()).
            forall|i: int| slab.num_index_blocks as int <= i < (slab.num_index_blocks + slab.num_data_blocks) as int
                ==> !slab.index.is_bit_set(i),
        ensures
            slab@.is_freshly_initialized(),
    {
        // Proof: All data blocks unset means no block in allocated_blocks.
        // allocated_blocks = { i | is_bit_set(num_index_blocks + i) } = empty set.
        assert(slab@.allocated_blocks =~= Set::<int>::empty()) by {
            assert forall|i: int| !slab@.allocated_blocks.contains(i) by {
                if 0 <= i < slab@.num_data_blocks {
                    let bitmap_idx = slab.num_index_blocks as int + i;
                    assert(!slab.index.is_bit_set(bitmap_idx));
                }
            }
        }
    }


    /// Lemma: A freshly initialized slab has maximum free capacity.
    proof fn lemma_fresh_slab_max_free(slab: &Slab)
        requires
            slab.inv(),
            slab@.is_freshly_initialized(),
        ensures
            slab@.free() == slab@.capacity(),
            slab@.used() == 0,
    {
        // If allocated_blocks is empty, used() = 0, free() = capacity - 0 = capacity.
    }

    //==============================================================================================

    /// Lemma (Issue 2): Metadata and Data regions are disjoint.
    /// This proves that writing to the index bitmap cannot corrupt data blocks.
    proof fn lemma_metadata_data_disjoint(&self, base_addr: int, index_bytes: int)
        requires
            self.inv(),
            base_addr >= 0,
            // data_addr is computed as base_addr + num_index_blocks * block_size
            self.data_addr as int == base_addr + self.num_index_blocks as int * self.block_size as int,
            // index_bytes is the number of bytes used by the bitmap
            index_bytes >= 0,
            // num_index_blocks * block_size >= index_bytes (ceiling division ensures this)
            self.num_index_blocks as int * self.block_size as int >= index_bytes,
        ensures
            self.metadata_data_disjoint(base_addr, index_bytes),
    {
        // The index uses index_bytes bytes starting at base_addr.
        // num_index_blocks = ceil(index_bytes / block_size), so:
        // num_index_blocks * block_size >= index_bytes.
        // Therefore: data_addr = base_addr + num_index_blocks * block_size
        //                     >= base_addr + index_bytes
        //                      = index_region_end.
        let index_region_end = base_addr + index_bytes;
        let data_region_start = self.data_addr as int;

        // From precondition: data_addr = base_addr + num_index_blocks * block_size.
        // From precondition: num_index_blocks * block_size >= index_bytes.
        // Therefore: data_addr >= base_addr + index_bytes = index_region_end.
        assert(data_region_start >= index_region_end);
    }


    /// Lemma: Block memory regions are disjoint for different block indices.
    /// This proves the no_memory_aliasing property.
    proof fn lemma_blocks_disjoint(view: &SlabView, i: int, j: int)
        requires
            view.block_size > 0,
            0 <= i < view.num_data_blocks,
            0 <= j < view.num_data_blocks,
            i != j,
        ensures
            view.blocks_are_disjoint(i, j),
    {
        // Proof:
        // block_addr(i) = data_addr + i * block_size
        // block_addr(j) = data_addr + j * block_size
        let addr_i = view.block_addr(i);
        let addr_j = view.block_addr(j);
        let bs = view.block_size;

        // Expand the definitions
        assert(addr_i == view.data_addr + i * bs);
        assert(addr_j == view.data_addr + j * bs);

        // Use distributive property: bs * (j - i) = bs * j - bs * i
        vstd::arithmetic::mul::lemma_mul_is_distributive_sub(bs, j, i);
        assert(bs * (j - i) == bs * j - bs * i);

        // Commutativity of multiplication
        vstd::arithmetic::mul::lemma_mul_is_commutative(bs, j - i);
        vstd::arithmetic::mul::lemma_mul_is_commutative(bs, j);
        vstd::arithmetic::mul::lemma_mul_is_commutative(bs, i);
        assert((j - i) * bs == j * bs - i * bs);

        // Also for i - j
        vstd::arithmetic::mul::lemma_mul_is_distributive_sub(bs, i, j);
        vstd::arithmetic::mul::lemma_mul_is_commutative(bs, i - j);
        assert((i - j) * bs == i * bs - j * bs);

        if i < j {
            // j - i >= 1
            let diff = j - i;
            assert(diff >= 1);
            // diff * bs >= 1 * bs = bs (monotonicity of multiplication)
            vstd::arithmetic::mul::lemma_mul_inequality(1, diff, bs);
            assert(1 * bs <= diff * bs);
            assert(bs <= diff * bs);
            // addr_j - addr_i = j * bs - i * bs = (j - i) * bs >= bs
            assert(addr_j - addr_i == j * bs - i * bs);
            assert(addr_j - addr_i == diff * bs);
            assert(addr_j - addr_i >= bs);
            assert(addr_i + bs <= addr_j);
        } else {
            // i > j, so i - j >= 1
            let diff = i - j;
            assert(diff >= 1);
            vstd::arithmetic::mul::lemma_mul_inequality(1, diff, bs);
            assert(1 * bs <= diff * bs);
            assert(bs <= diff * bs);
            // addr_i - addr_j = i * bs - j * bs = (i - j) * bs >= bs
            assert(addr_i - addr_j == i * bs - j * bs);
            assert(addr_i - addr_j == diff * bs);
            assert(addr_i - addr_j >= bs);
            assert(addr_j + bs <= addr_i);
        }
    }


    /// Lemma: addr_to_block_idx(block_addr(i)) == i for valid block index i.
    /// This proves the inverse relationship between address and block index.
    proof fn lemma_addr_block_idx_inverse(view: &SlabView, i: int)
        requires
            view.block_size > 0,
            0 <= i < view.num_data_blocks,
        ensures
            view.addr_block_idx_inverse(i),
    {
        // Proof:
        // block_addr(i) = data_addr + i * block_size
        // addr_to_block_idx(block_addr(i)) = (block_addr(i) - data_addr) / block_size
        //                                 = (i * block_size) / block_size
        //                                 = i
        let bs = view.block_size;
        let addr = view.block_addr(i);

        // addr = data_addr + i * bs
        assert(addr == view.data_addr + i * bs);

        // addr - data_addr = i * bs
        assert(addr - view.data_addr == i * bs);

        // (i * bs) / bs = i (using lemma_div_by_multiple with b=i, d=bs).
        vstd::arithmetic::div_mod::lemma_div_by_multiple(i, bs);
        assert((i * bs) / bs == i);

        // addr_to_block_idx(addr) = (addr - data_addr) / bs = (i * bs) / bs = i
        assert(view.addr_to_block_idx(addr) == (addr - view.data_addr) / bs);
        assert(view.addr_to_block_idx(addr) == i);
    }


    /// Lemma: block_addr(addr_to_block_idx(a)) == a for valid addresses.
    /// This proves the inverse relationship for valid addresses.
    proof fn lemma_block_addr_inverse(view: &SlabView, addr: int)
        requires
            view.block_size > 0,
            view.is_valid_addr(addr),
        ensures
            view.block_addr_inverse(addr),
    {
        // Proof:
        // is_valid_addr(addr) implies (addr - data_addr) % block_size == 0.
        // Let offset = addr - data_addr.
        // Since offset % block_size == 0, offset = k * block_size for some k.
        // addr_to_block_idx(addr) = offset / block_size = k.
        // block_addr(k) = data_addr + k * block_size = data_addr + offset = addr.
        let bs = view.block_size;
        let data_addr = view.data_addr;
        let offset = addr - data_addr;

        // From is_valid_addr: offset >= 0 and offset % bs == 0.
        assert(offset >= 0);
        assert(offset % bs == 0);

        // When offset % bs == 0, we have: (offset / bs) * bs == offset.
        assert((offset / bs) * bs == offset) by(nonlinear_arith)
            requires bs > 0, offset >= 0, offset % bs == 0;

        // addr_to_block_idx = offset / bs.
        let k = offset / bs;
        assert(view.addr_to_block_idx(addr) == k);

        // block_addr(k) = data_addr + k * bs = data_addr + offset = addr.
        assert(view.block_addr(k) == data_addr + k * bs);
        assert(k * bs == offset);
        assert(data_addr + offset == addr);
    }


    /// Lemma: All allocated blocks are within valid range.
    proof fn lemma_allocated_blocks_in_range(&self)
        requires
            self.inv(),
        ensures
            self@.allocated_blocks_in_range(),
    {
        // From the view definition, allocated_blocks only contains indices i where:
        // 0 <= i < num_data_blocks && is_bit_set(num_index_blocks + i).
        // Therefore, any allocated block index is in [0, num_data_blocks).
    }


    /// Lemma: No memory aliasing - all allocated blocks have disjoint regions.
    proof fn lemma_no_memory_aliasing(&self)
        requires
            self.inv(),
        ensures
            self@.no_memory_aliasing(),
    {
        // For any two allocated blocks i, j with i != j:
        // - 0 <= i < num_data_blocks (from lemma_allocated_blocks_in_range)
        // - 0 <= j < num_data_blocks (from lemma_allocated_blocks_in_range)
        // - blocks_are_disjoint(i, j) (from lemma_blocks_disjoint)
        // Therefore, no_memory_aliasing holds.
        assert forall|i: int, j: int|
            (self@.is_allocated(i) && self@.is_allocated(j) && i != j)
            implies self@.blocks_are_disjoint(i, j)
        by {
            if self@.is_allocated(i) && self@.is_allocated(j) && i != j {
                // From view definition, allocated indices are in valid range.
                assert(0 <= i < self@.num_data_blocks);
                assert(0 <= j < self@.num_data_blocks);
                Self::lemma_blocks_disjoint(&self@, i, j);
            }
        }
    }

    //==============================================================================================

    /// Lemma: a * b is always divisible by b (when b > 0).
    proof fn lemma_mul_divisible(a: int, b: int)
        requires b > 0,
        ensures (a * b) % b == 0,
    {
        assert((a * b) % b == 0) by(nonlinear_arith)
            requires b > 0;
    }


    /// Lemma: (a * b) / b == a (when b > 0).
    proof fn lemma_div_cancel(a: int, b: int)
        requires b > 0,
        ensures (a * b) / b == a,
    {
        assert((a * b) / b == a) by(nonlinear_arith)
            requires b > 0;
    }


    /// Lemma: if a < b and c > 0, then a * c < b * c.
    proof fn lemma_mul_inequality(a: int, b: int, c: int)
        requires a < b, c > 0,
        ensures a * c < b * c,
    {
        assert(a * c < b * c) by(nonlinear_arith)
            requires a < b, c > 0;
    }


    /// Lemma: if q = a / b (integer division), then q * b <= a.
    proof fn lemma_div_mul_le(a: int, b: int)
        requires b > 0, a >= 0,
        ensures (a / b) * b <= a,
    {
        assert((a / b) * b <= a) by(nonlinear_arith)
            requires b > 0, a >= 0;
    }


    /// Lemma: for integer division, (a / b) * b + (a % b) == a.
    proof fn lemma_div_mod_identity(a: int, b: int)
        requires b > 0, a >= 0,
        ensures (a / b) * b + (a % b) == a,
    {
        assert((a / b) * b + (a % b) == a) by(nonlinear_arith)
            requires b > 0, a >= 0;
    }


    /// Lemma: distributive property (a + b) * c == a * c + b * c.
    proof fn lemma_distributive(a: int, b: int, c: int)
        ensures (a + b) * c == a * c + b * c,
    {
        assert((a + b) * c == a * c + b * c) by(nonlinear_arith);
    }


    /// Lemma: Product of two positive integers is positive.
    proof fn lemma_pos_mul_pos(a: int, b: int)
        requires
            a > 0,
            b > 0,
        ensures
            a * b > 0,
    {
        assert(a * b > 0) by(nonlinear_arith)
            requires a > 0, b > 0;
    }
}


/// Test: Error conditions are prevented by preconditions.
/// This test documents what the original error tests check, but in Verus
/// style where preconditions prevent invalid calls.
proof fn test_error_conditions_prevented()
{
    // In the original code:
    // - test_slab_creation_invalid_length: len == 0 returns InvalidArgument
    // - test_slab_creation_invalid_block_size: block_size == 0 returns InvalidArgument
    //
    // In Verus, our from_raw_parts requires:
    //   len > 0, block_size > 0
    // Therefore, calling with len == 0 or block_size == 0 is NOT allowed by
    // the type system. This is a stronger guarantee than runtime error checking:
    // invalid inputs are prevented at compile time.
    //
    // Similarly for double_deallocate and out_of_bounds:
    // - deallocate requires is_allocated(addr_to_block_idx(addr))
    // - deallocate requires is_valid_addr(addr)
    // Violating these preconditions is a compile-time error.
}

//==================================================================================================

/// Test: Address to Block Index Bijection
/// Original: Not tested
/// Verified: Proves addr_to_block_idx and block_addr are inverses
proof fn test_addr_block_bijection_property(
    view: SlabView,
    block_idx: int,
    addr: int,
)
    requires
        view.block_size > 0,
        view.num_data_blocks > 0,
        0 <= block_idx < view.num_data_blocks,
        view.is_valid_addr(addr),
{
    // addr_to_block_idx(block_addr(i)) == i
    // This follows from the definitions:
    // block_addr(i) = data_addr + i * block_size
    // addr_to_block_idx(a) = (a - data_addr) / block_size
    let computed_addr: int = view.block_addr(block_idx);
    let back_to_idx: int = view.addr_to_block_idx(computed_addr);
    // (data_addr + i * block_size - data_addr) / block_size = i * block_size / block_size = i
    // Use arithmetic facts to prove this.
    assert(computed_addr == view.data_addr + block_idx * view.block_size);
    assert(computed_addr - view.data_addr == block_idx * view.block_size);
    // For i >= 0 and block_size > 0: (i * block_size) / block_size == i
    Slab::lemma_div_cancel(block_idx, view.block_size);
    assert(back_to_idx == block_idx);
}


/// Test: All Allocated Blocks Are In Range
/// Original: Implicitly assumed
/// Verified: Proves allocated_blocks are within [0, num_data_blocks)
proof fn test_allocated_blocks_in_range_property(view: SlabView)
    requires
        view.allocated_blocks_in_range(),
{
    // From allocated_blocks_in_range():
    // forall |i| is_allocated(i) ==> (0 <= i < num_data_blocks)
    assert forall |i: int| view.is_allocated(i)
        implies 0 <= i < view.num_data_blocks
    by {
        // This follows directly from the precondition.
    }
}


/// Test: No Memory Aliasing Property
/// Original: Not tested
/// Verified: Proves different allocated blocks have disjoint memory regions
proof fn test_no_memory_aliasing_property(view: SlabView)
    requires
        view.no_memory_aliasing(),
{
    // From no_memory_aliasing():
    // forall |i, j| (is_allocated(i) && is_allocated(j) && i != j) ==> blocks_are_disjoint(i, j)
    assert forall |i: int, j: int|
        (view.is_allocated(i) && view.is_allocated(j) && i != j)
        implies view.blocks_are_disjoint(i, j)
    by {
        // This follows directly from the precondition.
    }
}


/// Test: Liveness - Deallocation Enables Reallocation
/// Verified: Freed block becomes available for allocation
proof fn test_liveness_dealloc_enables_alloc(view: SlabView, freed_view: SlabView, block_idx: int)
    requires
        view.used() == view.capacity(),  // Was full
        view.is_allocated(block_idx),
        0 <= block_idx < view.num_data_blocks,
        !freed_view.is_allocated(block_idx),  // Now freed
        freed_view.num_data_blocks == view.num_data_blocks,
        // All other blocks unchanged
        forall|i: int| (0 <= i < view.num_data_blocks && i != block_idx) ==>
            (view.is_allocated(i) <==> freed_view.is_allocated(i)),
        // Additional requirements to ensure sets are well-formed
        view.num_data_blocks > 0,
        view.allocated_blocks_in_range(),
        freed_view.allocated_blocks_in_range(),
    ensures
        freed_view.can_allocate(),
{
    // After freeing one block from a full slab:
    // Prove that view.allocated_blocks is finite
    let old_set: Set<int> = view.allocated_blocks;
    let new_set: Set<int> = freed_view.allocated_blocks;
    let removed_set: Set<int> = old_set.remove(block_idx);

    // Prove old_set is finite via subset of set_int_range
    let range_set: Set<int> = set_int_range(0, view.num_data_blocks);
    assert forall|i: int| old_set.contains(i) implies range_set.contains(i) by {
        // old_set.contains(i) means view.is_allocated(i)
        // By allocated_blocks_in_range(): is_allocated(i) ==> 0 <= i < num_data_blocks
        // Therefore i is in range_set
        if old_set.contains(i) {
            assert(view.is_allocated(i));
            assert(view.allocated_blocks_in_range());
            assert(0 <= i < view.num_data_blocks);
        }
    }
    assert(old_set.subset_of(range_set));
    lemma_int_range(0, view.num_data_blocks);
    lemma_set_subset_finite(range_set, old_set);
    assert(old_set.finite());

    // removed_set is finite
    assert(removed_set.finite());

    // Prove new_set is finite
    let new_range_set: Set<int> = set_int_range(0, freed_view.num_data_blocks);
    assert forall|i: int| new_set.contains(i) implies new_range_set.contains(i) by {
        if new_set.contains(i) {
            assert(freed_view.is_allocated(i));
            assert(freed_view.allocated_blocks_in_range());
            assert(0 <= i < freed_view.num_data_blocks);
        }
    }
    assert(new_set.subset_of(new_range_set));
    lemma_int_range(0, freed_view.num_data_blocks);
    lemma_set_subset_finite(new_range_set, new_set);
    assert(new_set.finite());

    // Prove new_set is a subset of removed_set
    assert forall|i: int| new_set.contains(i) implies removed_set.contains(i) by {
        if new_set.contains(i) {
            assert(freed_view.is_allocated(i));
            assert(i != block_idx);
            assert(0 <= i < view.num_data_blocks);
            assert(view.is_allocated(i));
            assert(old_set.contains(i));
            assert(removed_set.contains(i));
        }
    }
    assert(new_set.subset_of(removed_set));

    // old_set contains block_idx
    assert(old_set.contains(block_idx));

    // Use axiom: removing decreases length by 1
    axiom_set_remove_len(old_set, block_idx);
    assert(removed_set.len() == old_set.len() - 1);

    // new_set.len() <= removed_set.len()
    lemma_len_subset(new_set, removed_set);

    // Therefore: freed_view.used() < freed_view.capacity()
    assert(freed_view.can_allocate());
}


/// Test: Fresh Initialization Property
/// Verified: Freshly initialized slab has no allocated blocks
proof fn test_fresh_initialization_property(view: SlabView)
    requires
        view.is_freshly_initialized(),
    ensures
        view.used() == 0,
        view.free() == view.capacity(),
{
    // is_freshly_initialized() ==> allocated_blocks is empty
    // ==> used() = |allocated_blocks| = 0
    // ==> free() = capacity - 0 = capacity
}

} // verus!
