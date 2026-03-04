// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

impl Slab {
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
            slab.data_addr as int > 0,
            // Memory bounds conditions.
            (slab.num_data_blocks as int) * (slab.block_size as int) <= usize::MAX as int,
            (slab.data_addr as int) + (slab.num_data_blocks as int) * (slab.block_size as int) <= usize::MAX as int,
            // Metadata/Data disjointness condition.
            slab.data_addr as int >= slab.num_index_blocks as int * slab.block_size as int,
            // Power-of-two and alignment conditions.
            is_pow2(slab.block_size as int),
            slab.data_addr as int % slab.block_size as int == 0,
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
    //==============================================================================================

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
    /// Lemma: distributive property (a + b) * c == a * c + b * c.
    proof fn lemma_distributive(a: int, b: int, c: int)
        ensures (a + b) * c == a * c + b * c,
    {
        assert((a + b) * c == a * c + b * c) by(nonlinear_arith);
    }
    /// Trusted bridge: bitwise check `n & (n - 1) == 0` implies `is_pow2(n)`.
    ///
    /// # Trust Justification
    ///
    /// Verus does not natively support bitwise operation reasoning.
    /// This is a well-known mathematical property: for n > 0,
    /// n & (n - 1) == 0 iff n is a power of two. See Hacker's Delight, Chapter 2.
    #[verifier::external_body]
    proof fn lemma_bitwise_implies_is_pow2(n: usize)
        requires n > 0, n & sub(n, 1) == 0,
        ensures is_pow2(n as int),
    {}

    /// Proves that after a failed clear, slab invariant is preserved.
    proof fn lemma_dealloc_clear_err_preserves_inv(slab: &Slab, old_slab: &Slab)
        requires
            old_slab.inv(),
            slab.index.inv(),
            slab.index@.set_bits =~= old_slab.index@.set_bits,
            slab.index@.number_of_bits() == old_slab.index@.number_of_bits(),
            slab.num_index_blocks == old_slab.num_index_blocks,
            slab.num_data_blocks == old_slab.num_data_blocks,
            slab.block_size == old_slab.block_size,
            slab.data_addr == old_slab.data_addr,
        ensures
            slab.inv(),
    {
        assert forall|j: int| 0 <= j < slab.num_index_blocks as int
            implies slab.index.is_bit_set(j) by {
            assert(old_slab.index.is_bit_set(j));
        }
        Self::lemma_inv_from_components(slab);
    }

    //==============================================================================================
    // Extracted proof-block lemmas
    //==============================================================================================

    /// Lemma: Proves layout bounds during slab construction.
    /// Establishes that num_index_blocks >= 1, num_data_blocks > 0,
    /// and the product num_index_blocks * block_size is bounded.
    proof fn lemma_from_raw_parts_layout_bounds(
        len: usize, block_size: usize, total_num_blocks: usize, index_len: usize,
        num_index_blocks: usize, num_data_blocks: usize, addr: usize,
    )
        requires
            len > 0,
            len < i32::MAX as usize,
            block_size > 0,
            block_size <= len,
            addr > 0,
            addr as int + len as int <= usize::MAX as int,
            total_num_blocks == len / block_size,
            total_num_blocks >= 8,
            index_len == total_num_blocks / 8,
            num_index_blocks == index_len / block_size
                + (if index_len % block_size == 0 { 0usize } else { 1usize }),
            num_index_blocks <= total_num_blocks,
            num_data_blocks == total_num_blocks - num_index_blocks,
        ensures
            num_index_blocks >= 1,
            num_data_blocks > 0,
            (num_index_blocks as int) * (block_size as int) < (total_num_blocks as int) * (block_size as int),
            (total_num_blocks as int) * (block_size as int) <= len as int,
            (addr as int) + (num_index_blocks as int) * (block_size as int) <= (addr as int) + (len as int),
            (addr as int) + (num_index_blocks as int) * (block_size as int) <= usize::MAX as int,
    {
        // index_len >= 1 since total_num_blocks >= 8.
        assert(index_len >= 1);
        // Prove num_index_blocks >= 1.
        if index_len >= block_size {
            assert(index_len / block_size >= 1) by(nonlinear_arith)
                requires index_len >= block_size, block_size > 0int;
        } else {
            assert(index_len % block_size == index_len) by(nonlinear_arith)
                requires 0 < index_len < block_size;
            assert(index_len % block_size != 0);
        }
        assert(num_index_blocks >= 1);
        // Prove num_data_blocks > 0.
        assert(num_index_blocks <= index_len + 1) by {
            assert(index_len / block_size <= index_len) by(nonlinear_arith)
                requires block_size >= 1int, index_len >= 0int;
        }
        assert(index_len + 1 <= total_num_blocks) by {
            assert(total_num_blocks / 8 + 1 <= total_num_blocks) by(nonlinear_arith)
                requires total_num_blocks >= 8int;
        }
        assert(num_index_blocks < total_num_blocks);
        assert(num_data_blocks > 0);
        // Prove product bounds.
        Self::lemma_div_mul_le(len as int, block_size as int);
        Self::lemma_mul_inequality(num_index_blocks as int, total_num_blocks as int, block_size as int);
    }

    /// Lemma: Converts is_zero properties on a byte sequence to concrete equality with 0u8.
    proof fn lemma_raw_array_storage_zeroed(s: Seq<u8>)
        requires
            forall|i: int| 0 <= i < s.len() ==> is_zero(#[trigger] s[i]),
        ensures
            forall|i: int| 0 <= i < s.len() ==> s[i] == 0u8,
    {
        assert forall|i: int| 0 <= i < s.len() implies s[i] == 0u8 by {
            axiom_u8_zero_is_0(s[i]);
        }
    }

    /// Lemma: Establishes pre-loop invariants for from_raw_parts.
    /// Connects bitmap number_of_bits to total_num_blocks and layout.
    proof fn lemma_from_raw_parts_pre_loop(
        index_nbits: int, index_len: int, total_num_blocks: int,
        num_index_blocks: int, num_data_blocks: int,
    )
        requires
            index_nbits == index_len * 8,
            total_num_blocks == index_len * 8,
            num_index_blocks + num_data_blocks == total_num_blocks,
            num_index_blocks < total_num_blocks,
        ensures
            index_nbits == total_num_blocks,
            num_index_blocks + num_data_blocks == index_nbits,
    {
    }

    /// Lemma: Proves postconditions after the initialization loop in from_raw_parts.
    /// Establishes slab invariant, view fields, and emptiness.
    proof fn lemma_from_raw_parts_post_loop(
        slab: &Slab, addr: int, len: int, total_num_blocks: int,
    )
        requires
            slab.index.inv(),
            slab.block_size > 0,
            slab.num_data_blocks > 0,
            slab.num_index_blocks > 0,
            // All index blocks are set in the bitmap.
            forall|i: int| #![trigger slab.index@.set_bits.contains(i)]
                0 <= i < slab.num_index_blocks as int ==> slab.index@.set_bits.contains(i),
            // All data blocks are unset in the bitmap.
            forall|i: int| slab.num_index_blocks as int <= i < slab.index@.number_of_bits()
                ==> !slab.index.is_bit_set(i),
            // Layout relationships.
            (slab.num_index_blocks as int) + (slab.num_data_blocks as int) == slab.index@.number_of_bits(),
            total_num_blocks == slab.index@.number_of_bits(),
            (slab.num_data_blocks as int) < total_num_blocks,
            // Address computation.
            (slab.data_addr as int) == addr + (slab.num_index_blocks as int) * (slab.block_size as int),
            addr > 0,
            // Original preconditions.
            len > 0,
            len < i32::MAX as int,
            slab.block_size as int <= len,
            addr + len <= usize::MAX as int,
            total_num_blocks == len / slab.block_size as int,
            // Power of two and alignment.
            is_pow2(slab.block_size as int),
            slab.data_addr as int % slab.block_size as int == 0,
        ensures
            slab.inv(),
            slab@.block_size == slab.block_size as int,
            slab@.allocated_blocks =~= Set::<int>::empty(),
            slab@.data_addr > addr,
            slab@.data_addr % slab.block_size as int == 0,
            slab@.num_data_blocks > 0,
            slab@.data_addr + slab@.num_data_blocks * slab@.block_size <= addr + len,
    {
        // Prove memory bounds.
        Self::lemma_div_mul_le(len, slab.block_size as int);
        assert(total_num_blocks == len / slab.block_size as int);
        assert(total_num_blocks * slab.block_size as int <= len);
        Self::lemma_mul_inequality(
            slab.num_data_blocks as int, total_num_blocks, slab.block_size as int,
        );
        assert((slab.num_data_blocks as int) * (slab.block_size as int)
            < total_num_blocks * slab.block_size as int);
        assert(len < usize::MAX as int);
        assert((slab.num_data_blocks as int) * (slab.block_size as int) <= usize::MAX as int);

        // Prove data_addr + num_data_blocks * block_size <= addr + len.
        assert(slab.data_addr as int
            == addr + slab.num_index_blocks as int * slab.block_size as int);
        Self::lemma_distributive(
            slab.num_index_blocks as int, slab.num_data_blocks as int,
            slab.block_size as int,
        );
        assert(slab.num_index_blocks as int * slab.block_size as int
            + slab.num_data_blocks as int * slab.block_size as int
            == (slab.num_index_blocks as int + slab.num_data_blocks as int)
                * slab.block_size as int);
        assert(slab.data_addr as int + slab.num_data_blocks as int * slab.block_size as int
            == addr + (slab.num_index_blocks as int + slab.num_data_blocks as int)
                * slab.block_size as int);
        assert(slab.num_index_blocks as int + slab.num_data_blocks as int == total_num_blocks);
        assert(slab.data_addr as int + slab.num_data_blocks as int * slab.block_size as int
            == addr + total_num_blocks * slab.block_size as int);
        assert(total_num_blocks * slab.block_size as int <= len);
        assert(addr + total_num_blocks * slab.block_size as int <= addr + len);
        assert(addr + len <= usize::MAX as int);
        assert(slab.data_addr as int + slab.num_data_blocks as int * slab.block_size as int
            <= usize::MAX as int);

        // Prove metadata/data disjointness.
        assert(slab.data_addr as int
            == addr + slab.num_index_blocks as int * slab.block_size as int);
        assert(addr > 0);
        assert(slab.data_addr as int
            > slab.num_index_blocks as int * slab.block_size as int);
        assert(slab.data_addr as int
            >= slab.num_index_blocks as int * slab.block_size as int);

        // Prove inv().
        Self::lemma_inv_from_components(slab);
        Self::lemma_view_fields(slab);
        Self::lemma_new_slab_is_empty(slab);
        // Prove allocated_blocks =~= Set::empty() from !is_allocated for all data blocks.
        assert(slab@.allocated_blocks =~= Set::<int>::empty()) by {
            assert forall|i: int| !slab@.allocated_blocks.contains(i) by {
                if 0 <= i < slab@.num_data_blocks {
                    assert(!slab@.is_allocated(i));
                } else {
                    assert(slab@.allocated_blocks_in_range());
                    assert(!slab@.is_allocated(i));
                }
            }
        }
        assert(slab.data_addr as int > addr);
        assert(slab.data_addr as int % slab.block_size as int == 0);
        assert(slab@.data_addr + slab@.num_data_blocks * slab@.block_size
            <= addr + len);
    }

    /// Lemma: When bitmap alloc fails, slab state is preserved and cannot allocate.
    proof fn lemma_alloc_error_preserves_state(slab: &Slab, old_slab: &Slab)
        requires
            old_slab.inv(),
            slab.index.inv(),
            slab.index@.set_bits =~= old_slab.index@.set_bits,
            slab.index@.number_of_bits() == old_slab.index@.number_of_bits(),
            slab.num_index_blocks == old_slab.num_index_blocks,
            slab.num_data_blocks == old_slab.num_data_blocks,
            slab.block_size == old_slab.block_size,
            slab.data_addr == old_slab.data_addr,
            !slab.index@.has_free_bit(),
        ensures
            slab.inv(),
            slab@ == old_slab@,
            !old_slab@.can_allocate(),
    {
        // Liveness: if can_allocate(), bitmap has_free_bit - contradiction.
        if old_slab@.can_allocate() {
            old_slab.lemma_can_allocate_implies_bitmap_has_free_bit();
            assert(false);
        }
        // Prove index blocks still set.
        assert forall|i: int| 0 <= i < slab.num_index_blocks as int
            implies #[trigger] slab.index@.set_bits.contains(i) by {
            assert(old_slab.index@.set_bits.contains(i));
            assert(slab.index@.set_bits.contains(i)
                == old_slab.index@.set_bits.contains(i));
        }
        // Prove inv().
        Self::lemma_inv_from_components(slab);
        // Prove view equality.
        assert(slab@.num_data_blocks == old_slab@.num_data_blocks);
        assert(slab@.block_size == old_slab@.block_size);
        assert(slab@.data_addr == old_slab@.data_addr);
        assert(slab@.allocated_blocks =~= old_slab@.allocated_blocks) by {
            assert forall|j: int| 0 <= j < slab.num_data_blocks as int implies
                (slab@.allocated_blocks.contains(j)
                    == old_slab@.allocated_blocks.contains(j)) by {
                let bitmap_idx: int = slab.num_index_blocks as int + j;
                assert(slab.index@.set_bits.contains(bitmap_idx)
                    == old_slab.index@.set_bits.contains(bitmap_idx));
            }
        }
        assert(slab@ == old_slab@);
    }

    /// Lemma: After bitmap alloc, the returned bit is a data block with valid bounds.
    proof fn lemma_alloc_block_is_data_block_with_bounds(
        slab: &Slab, old_slab: &Slab, block: int,
    )
        requires
            old_slab.inv(),
            slab.inv(),
            // Bitmap alloc postconditions.
            0 <= block < old_slab.index@.number_of_bits(),
            !old_slab.index.is_bit_set(block),
            slab.index.is_bit_set(block),
            // Fields unchanged.
            slab.num_index_blocks == old_slab.num_index_blocks,
            slab.num_data_blocks == old_slab.num_data_blocks,
            slab.block_size == old_slab.block_size,
            slab.data_addr == old_slab.data_addr,
            slab.index@.number_of_bits() == old_slab.index@.number_of_bits(),
        ensures
            block >= slab.num_index_blocks as int,
            block < (slab.num_index_blocks + slab.num_data_blocks) as int,
    {
        // An unset bit cannot be an index block (inv says index blocks are always set).
        assert(block >= slab.num_index_blocks as int) by {
            if block < slab.num_index_blocks as int {
                assert(old_slab.index.is_bit_set(block));
            }
        };
        // Bounds: bitmap length equals index + data blocks.
        assert(old_slab.num_index_blocks as int + old_slab.num_data_blocks as int
            == slab.index@.number_of_bits());
        assert(block < slab.index@.number_of_bits());
        assert(block < (slab.num_index_blocks + slab.num_data_blocks) as int);
    }

    /// Lemma: block_idx * block_size and data_addr + block_idx * block_size fit in usize.
    proof fn lemma_alloc_product_in_bounds(slab: &Slab, block_idx: int)
        requires
            slab.inv(),
            0 <= block_idx < (slab.num_data_blocks as int),
        ensures
            block_idx * (slab.block_size as int) < (slab.num_data_blocks as int) * (slab.block_size as int),
            block_idx * (slab.block_size as int) <= usize::MAX as int,
            (slab.data_addr as int) + block_idx * (slab.block_size as int) <= usize::MAX as int,
    {
        Self::lemma_mul_inequality(
            block_idx, slab.num_data_blocks as int, slab.block_size as int,
        );
    }

    /// Lemma: Establishes allocate postconditions (address validity, block index, frame).
    proof fn lemma_alloc_establishes_postconditions(
        slab: &Slab, old_slab: &Slab, block: int,
        block_idx: int, block_addr: int,
    )
        requires
            old_slab.inv(),
            slab.inv(),
            // Block relationships.
            block_idx == block - (slab.num_index_blocks as int),
            0 <= block_idx < (slab.num_data_blocks as int),
            block_addr == (slab.data_addr as int) + block_idx * (slab.block_size as int),
            // Bitmap postconditions.
            slab.index.is_bit_set((slab.num_index_blocks as int) + block_idx),
            !old_slab.index.is_bit_set((slab.num_index_blocks as int) + block_idx),
            // Only the allocated bit changed.
            forall|k: int| k != block && 0 <= k < slab.index@.number_of_bits() ==>
                slab.index.is_bit_set(k) == old_slab.index.is_bit_set(k),
            // Fields unchanged.
            slab.num_index_blocks == old_slab.num_index_blocks,
            slab.num_data_blocks == old_slab.num_data_blocks,
            slab.block_size == old_slab.block_size,
            slab.data_addr == old_slab.data_addr,
        ensures
            old_slab@.is_valid_addr(block_addr),
            old_slab@.addr_to_block_idx(block_addr) == block_idx,
            !old_slab@.is_allocated(block_idx),
            slab@.is_allocated(block_idx),
            slab@.num_data_blocks == old_slab@.num_data_blocks,
            slab@.block_size == old_slab@.block_size,
            slab@.data_addr == old_slab@.data_addr,
            slab@.allocated_blocks =~= old_slab@.allocated_blocks.insert(block_idx),
            block_addr > 0,
    {
        Self::lemma_view_fields(old_slab);
        let bs: int = slab.block_size as int;
        let ndb: int = slab.num_data_blocks as int;

        // Address validity proofs.
        assert(block_addr == slab.data_addr as int + block_idx * bs);
        Self::lemma_mul_inequality(block_idx, ndb, bs);
        assert(block_addr < slab.data_addr as int + ndb * bs);
        Self::lemma_mul_divisible(block_idx, bs);
        assert((block_addr - slab.data_addr as int) % bs == 0);
        assert(old_slab@.is_valid_addr(block_addr));

        // Block index computation.
        Self::lemma_div_cancel(block_idx, bs);
        assert(old_slab@.addr_to_block_idx(block_addr) == block_idx);

        // Allocation status.
        assert(!old_slab@.is_allocated(block_idx));
        assert(slab@.is_allocated(block_idx));

        // Other bits unchanged.
        assert forall|i: int| 0 <= i < ndb && i != block_idx implies
            #[trigger] slab.index.is_bit_set(slab.num_index_blocks as int + i)
                == #[trigger] old_slab.index.is_bit_set(slab.num_index_blocks as int + i)
        by {
            let global_idx: int = slab.num_index_blocks as int + i;
            if global_idx == block {
                assert(i == block_idx);
            }
        }
        // Frame for allocated_blocks.
        assert forall|i: int| 0 <= i < ndb && i != block_idx
            implies slab@.is_allocated(i) == old_slab@.is_allocated(i) by {
            let bitmap_idx: int = slab.num_index_blocks as int + i;
            assert(slab.index.is_bit_set(bitmap_idx) == old_slab.index.is_bit_set(bitmap_idx));
        }
        // Prove allocated_blocks =~= old_allocated_blocks.insert(block_idx).
        assert(slab@.allocated_blocks =~= old_slab@.allocated_blocks.insert(block_idx)) by {
            assert forall|j: int| #![auto] slab@.allocated_blocks.contains(j)
                == old_slab@.allocated_blocks.insert(block_idx).contains(j) by {
                if j == block_idx {
                    assert(slab@.is_allocated(block_idx));
                } else if 0 <= j < ndb {
                    assert(slab@.is_allocated(j) == old_slab@.is_allocated(j));
                } else {
                    assert(slab@.allocated_blocks_in_range());
                    assert(!slab@.is_allocated(j));
                    assert(old_slab@.allocated_blocks_in_range());
                    assert(!old_slab@.is_allocated(j));
                }
            }
        }
    }

    /// Lemma: Proves offset and index bounds for deallocate.
    proof fn lemma_dealloc_offset_bounds(slab: &Slab, ptr: int)
        requires
            slab.inv(),
            slab@.is_valid_addr(ptr),
            slab@.can_deallocate(slab@.addr_to_block_idx(ptr)),
        ensures
            ptr >= (slab.data_addr as int),
            (ptr - (slab.data_addr as int)) >= 0,
            (ptr - (slab.data_addr as int)) < (slab.num_data_blocks as int) * (slab.block_size as int),
            ((ptr - (slab.data_addr as int)) % (slab.block_size as int)) == 0,
            ({
                let block_idx: int = (ptr - (slab.data_addr as int)) / (slab.block_size as int);
                &&& 0 <= block_idx < (slab.num_data_blocks as int)
                &&& (slab.num_index_blocks as int) + block_idx < slab.index@.number_of_bits()
                &&& (slab.num_index_blocks as int) + block_idx < usize::MAX as int
            }),
    {
        // From is_valid_addr.
        assert(ptr >= slab.data_addr as int);
        assert(ptr < slab.data_addr as int
            + slab.num_data_blocks as int * slab.block_size as int);
        assert((ptr - slab.data_addr as int) % slab.block_size as int == 0);

        let offset: int = ptr - slab.data_addr as int;
        assert(offset >= 0);
        assert(offset < slab.num_data_blocks as int * slab.block_size as int);

        let block_idx: int = offset / slab.block_size as int;
        assert(0 <= block_idx < slab.num_data_blocks as int);
        assert(slab.num_index_blocks as int + block_idx < slab.index@.number_of_bits());

        // Prove no overflow for usize computation.
        assert(slab.num_index_blocks as int + block_idx
            < slab.num_index_blocks as int + slab.num_data_blocks as int);
        assert(slab.num_index_blocks as int + slab.num_data_blocks as int
            == slab.index@.number_of_bits());
        Self::lemma_slab_inv_implies_bitmap_inv(slab);
        assert(slab.index@.number_of_bits() <= usize::MAX as int);
        assert(slab.num_index_blocks as int + block_idx < usize::MAX as int);
    }

    /// Lemma: Connects the computed index to addr_to_block_idx and proves the bit is set.
    proof fn lemma_dealloc_index_is_allocated(slab: &Slab, ptr: int, index: int)
        requires
            slab.inv(),
            slab@.is_valid_addr(ptr),
            slab@.can_deallocate(slab@.addr_to_block_idx(ptr)),
            index == (slab.num_index_blocks as int)
                + (ptr - (slab.data_addr as int)) / (slab.block_size as int),
            index < slab.index@.number_of_bits(),
        ensures
            slab.index.is_bit_set(index),
            slab@.is_allocated(slab@.addr_to_block_idx(ptr)),
            index == (slab.num_index_blocks as int) + slab@.addr_to_block_idx(ptr),
    {
        let block_idx_spec: int = slab@.addr_to_block_idx(ptr);
        assert(block_idx_spec
            == (ptr - slab.data_addr as int) / slab.block_size as int);
        assert(index == slab.num_index_blocks as int + block_idx_spec);
        assert(slab@.is_allocated(block_idx_spec));
        assert(slab.index.is_bit_set(index));
    }

    /// Lemma: After a successful clear in deallocate, proves inv, can_allocate, and frame.
    proof fn lemma_dealloc_clear_ok_postconditions(
        slab: &Slab, old_slab: &Slab, index: int, ptr: int,
    )
        requires
            old_slab.inv(),
            slab.index.inv(),
            slab.index@.number_of_bits() == old_slab.index@.number_of_bits(),
            // Fields unchanged.
            slab.num_index_blocks == old_slab.num_index_blocks,
            slab.num_data_blocks == old_slab.num_data_blocks,
            slab.block_size == old_slab.block_size,
            slab.data_addr == old_slab.data_addr,
            // Bitmap clear postconditions.
            !slab.index.is_bit_set(index),
            old_slab.index.is_bit_set(index),
            forall|j: int| j != index && 0 <= j < slab.index@.number_of_bits() ==>
                slab.index.is_bit_set(j) == old_slab.index.is_bit_set(j),
            // Block index relationships.
            index == (old_slab.num_index_blocks as int) + old_slab@.addr_to_block_idx(ptr),
            old_slab@.is_valid_addr(ptr),
            old_slab@.can_deallocate(old_slab@.addr_to_block_idx(ptr)),
        ensures
            slab.inv(),
            ({
                let block_idx_spec: int = old_slab@.addr_to_block_idx(ptr);
                &&& !slab@.is_allocated(block_idx_spec)
                &&& slab@.num_data_blocks == old_slab@.num_data_blocks
                &&& slab@.block_size == old_slab@.block_size
                &&& slab@.data_addr == old_slab@.data_addr
                &&& slab@.allocated_blocks =~=
                        old_slab@.allocated_blocks.remove(block_idx_spec)
                &&& slab@.can_allocate()
            }),
    {
        let block_idx_spec: int = old_slab@.addr_to_block_idx(ptr);
        assert(block_idx_spec >= 0);
        assert(index == slab.num_index_blocks as int + block_idx_spec);
        assert(index >= slab.num_index_blocks as int);

        // Prove all index blocks are still set (using set_bits trigger for lemma_inv_from_components).
        assert forall|j: int| 0 <= j < slab.num_index_blocks as int
            implies #[trigger] slab.index@.set_bits.contains(j) by {
            assert(j != index);
            assert(old_slab.index.is_bit_set(j));
            assert(slab.index.is_bit_set(j) == old_slab.index.is_bit_set(j));
            assert(slab.index.is_bit_set(j));
        }
        Self::lemma_inv_from_components(slab);

        // Prove can_allocate().
        assert(!slab.index.is_bit_set(index));
        slab.index.lemma_unset_bit_implies_has_free_bit(index);
        assert(slab.index@.has_free_bit());

        assert(0 <= block_idx_spec < slab@.num_data_blocks);
        assert(!slab@.is_allocated(block_idx_spec));

        // Prove allocated_blocks.len() < num_data_blocks.
        slab.lemma_allocated_blocks_finite();
        slab.lemma_allocated_blocks_subset_of_range();
        let full_range: Set<int> = set_int_range(0, slab@.num_data_blocks);
        lemma_int_range(0, slab@.num_data_blocks);

        // Witness: block_idx_spec is in full_range but not in allocated_blocks.
        assert(full_range.contains(block_idx_spec));
        assert(!slab@.allocated_blocks.contains(block_idx_spec));

        lemma_len_subset(slab@.allocated_blocks, full_range);
        assert(slab@.allocated_blocks.len() <= full_range.len());

        if slab@.allocated_blocks =~= full_range {
            assert(slab@.allocated_blocks.contains(block_idx_spec));
            assert(false);
        }
        assert(slab@.allocated_blocks !~= full_range);

        assert(slab@.allocated_blocks.len() < slab@.num_data_blocks) by {
            let with_witness: Set<int> =
                slab@.allocated_blocks.insert(block_idx_spec);
            assert forall|x: int| with_witness.contains(x)
                implies full_range.contains(x) by {
                if x == block_idx_spec {
                    assert(full_range.contains(block_idx_spec));
                } else {
                    assert(slab@.allocated_blocks.contains(x));
                    assert(full_range.contains(x));
                }
            }
            assert(with_witness.subset_of(full_range));
            axiom_set_insert_len(slab@.allocated_blocks, block_idx_spec);
            assert(with_witness.len() == slab@.allocated_blocks.len() + 1);
            lemma_len_subset(with_witness, full_range);
            assert(with_witness.len() <= full_range.len());
            assert(slab@.allocated_blocks.len() + 1 <= slab@.num_data_blocks);
        }

        assert(slab@.used() < slab@.capacity());
        assert(slab@.free() > 0);
        assert(slab@.can_allocate());

        // Prove allocated_blocks =~= old_allocated_blocks.remove(block_idx_spec).
        assert(slab@.allocated_blocks =~= old_slab@.allocated_blocks.remove(block_idx_spec)) by {
            assert forall|j: int| #![auto] slab@.allocated_blocks.contains(j)
                == old_slab@.allocated_blocks.remove(block_idx_spec).contains(j) by {
                if j == block_idx_spec {
                    assert(!slab@.is_allocated(block_idx_spec));
                } else if 0 <= j < slab@.num_data_blocks {
                    let bitmap_idx: int = slab.num_index_blocks as int + j;
                    assert(slab.index.is_bit_set(bitmap_idx)
                        == old_slab.index.is_bit_set(bitmap_idx));
                } else {
                    assert(slab@.allocated_blocks_in_range());
                    assert(!slab@.is_allocated(j));
                    assert(old_slab@.allocated_blocks_in_range());
                    assert(!old_slab@.is_allocated(j));
                }
            }
        }
    }
}
//==================================================================================================
// PointsToRaw Memory Permission Proof Helpers
//==================================================================================================

/// Lemma: The last block's range is a subset of the full range.
proof fn lemma_range_subset_last_block(base: int, block_size: int, n: int)
    requires
        block_size > 0,
        n >= 1,
    ensures
        set_int_range(base + (n - 1) * block_size, base + n * block_size)
            .subset_of(set_int_range(base, base + n * block_size)),
{
    let last: Set<int> = set_int_range(base + (n - 1) * block_size, base + n * block_size);
    let full: Set<int> = set_int_range(base, base + n * block_size);
    assert forall|x: int| last.contains(x) implies full.contains(x) by {
        assert(base + (n - 1) * block_size <= x < base + n * block_size);
        assert((n - 1) * block_size >= 0) by(nonlinear_arith)
            requires n >= 1, block_size > 0;
        assert(x >= base);
        assert(x < base + n * block_size);
    }
}

/// Lemma: Removing the last block from the full range gives the prefix range.
proof fn lemma_range_difference_last_block(base: int, block_size: int, n: int)
    requires
        block_size > 0,
        n >= 1,
    ensures
        set_int_range(base, base + n * block_size)
            .difference(set_int_range(base + (n - 1) * block_size, base + n * block_size))
            =~= set_int_range(base, base + (n - 1) * block_size),
{
    let full: Set<int> = set_int_range(base, base + n * block_size);
    let last: Set<int> = set_int_range(base + (n - 1) * block_size, base + n * block_size);
    let prefix: Set<int> = set_int_range(base, base + (n - 1) * block_size);
    let diff: Set<int> = full.difference(last);

    assert forall|x: int| diff.contains(x) <==> prefix.contains(x) by {
        if diff.contains(x) {
            assert(full.contains(x) && !last.contains(x));
            assert(base <= x < base + n * block_size);
            assert(!(base + (n - 1) * block_size <= x < base + n * block_size));
            assert(x < base + (n - 1) * block_size);
        }
        if prefix.contains(x) {
            assert(base <= x < base + (n - 1) * block_size);
            assert((n - 1) * block_size <= n * block_size) by(nonlinear_arith)
                requires block_size > 0, n >= 1;
            assert(x < base + n * block_size);
            assert(full.contains(x));
            assert((n - 1) * block_size < n * block_size) by(nonlinear_arith)
                requires block_size > 0, n >= 1;
            assert(!last.contains(x));
        }
    }
}

/// Splits a contiguous PointsToRaw into per-block permissions.
/// Given a permission for [base, base + n * block_size), returns a Map mapping
/// block index i to a permission for [base + i * block_size, base + (i+1) * block_size).
proof fn split_into_blocks(
    tracked perm: PointsToRaw,
    base: int,
    block_size: int,
    n: int,
) -> (tracked result: Map<int, PointsToRaw>)
    requires
        perm.is_range(base, n * block_size),
        block_size > 0,
        n >= 0,
    ensures
        forall|i: int| 0 <= i < n <==> result.dom().contains(i),
        forall|i: int| #![trigger result[i]]
            0 <= i < n ==>
                result[i].is_range(base + i * block_size, block_size)
                && result[i].provenance() == perm.provenance(),
    decreases n,
{
    if n == 0 {
        let tracked _padding = perm;
        Map::<int, PointsToRaw>::tracked_empty()
    } else {
        // Split off the last block (index n-1).
        let last_start: int = base + (n - 1) * block_size;
        let last_range: Set<int> = set_int_range(last_start, last_start + block_size);

        assert((n - 1) * block_size + block_size == n * block_size) by(nonlinear_arith)
            requires block_size > 0, n >= 1;

        lemma_range_subset_last_block(base, block_size, n);

        let tracked (last_perm, rest_perm) = perm.split(last_range);

        lemma_range_difference_last_block(base, block_size, n);

        let tracked mut result = split_into_blocks(rest_perm, base, block_size, n - 1);
        result.tracked_insert(n - 1, last_perm);
        result
    }
}

/// Joins per-block permissions back into a contiguous PointsToRaw.
/// Inverse of split_into_blocks.
/// NOTE: Currently unused. Reserved for future slab destruction / memory reclamation.
proof fn join_block_perms(
    tracked perms: Map<int, PointsToRaw>,
    base: int,
    block_size: int,
    n: int,
    prov: Provenance,
) -> (tracked result: PointsToRaw)
    requires
        block_size > 0,
        n >= 0,
        forall|i: int| 0 <= i < n <==> perms.dom().contains(i),
        forall|i: int| #![trigger perms[i]]
            0 <= i < n ==>
                perms[i].is_range(base + i * block_size, block_size)
                && perms[i].provenance() == prov,
    ensures
        result.is_range(base, n * block_size),
        result.provenance() == prov,
    decreases n,
{
    if n == 0 {
        assert(n * block_size == 0) by(nonlinear_arith)
            requires n == 0;
        PointsToRaw::empty(prov)
    } else {
        let tracked mut perms = perms;
        let tracked last_perm = perms.tracked_remove(n - 1);
        let tracked prefix_perm = join_block_perms(perms, base, block_size, n - 1, prov);
        let tracked result = prefix_perm.join(last_perm);

        let last_start: int = base + (n - 1) * block_size;
        assert((n - 1) * block_size + block_size == n * block_size) by(nonlinear_arith)
            requires block_size > 0, n >= 1;

        assert(set_int_range(base, base + (n - 1) * block_size)
            + set_int_range(last_start, last_start + block_size)
            =~= set_int_range(base, base + n * block_size)) by {
            let prefix_set: Set<int> = set_int_range(base, base + (n - 1) * block_size);
            let last_set: Set<int> = set_int_range(last_start, last_start + block_size);
            let full_set: Set<int> = set_int_range(base, base + n * block_size);
            assert forall|x: int|
                #![trigger full_set.contains(x)]
                (prefix_set + last_set).contains(x)
                <==> full_set.contains(x) by {
                assert((n - 1) * block_size >= 0) by(nonlinear_arith)
                    requires n >= 1, block_size > 0;
            }
        }

        result
    }
}
/// Lemma: After deallocating block_idx and inserting its permission back,
/// the map is well-formed for the new slab state.
proof fn lemma_dealloc_perms_wf(
    old_view: SlabView,
    new_view: SlabView,
    perms: Map<int, PointsToRaw>,
    block_idx: int,
    block_perm: PointsToRaw,
    prov: Provenance,
)
    requires
        old_view.block_size > 0,
        old_view.num_data_blocks > 0,
        old_view.perms_wf(perms, prov),
        0 <= block_idx < old_view.num_data_blocks,
        old_view.is_allocated(block_idx),
        block_perm.is_range(old_view.block_addr(block_idx), old_view.block_size),
        block_perm.provenance() == prov,
        new_view.num_data_blocks == old_view.num_data_blocks,
        new_view.block_size == old_view.block_size,
        new_view.data_addr == old_view.data_addr,
        new_view.allocated_blocks =~= old_view.allocated_blocks.remove(block_idx),
    ensures
        new_view.perms_wf(perms.insert(block_idx, block_perm), prov),
{
    let new_perms: Map<int, PointsToRaw> = perms.insert(block_idx, block_perm);

    // Prove new_view.perms_wf(new_perms, prov).
    // For free blocks in new_view:
    assert forall|i: int| #![trigger new_perms.dom().contains(i)]
        (0 <= i < new_view.num_data_blocks && !new_view.is_allocated(i))
        implies new_perms.dom().contains(i)
            && (#[trigger] new_perms[i]).is_range(
                new_view.block_addr(i), new_view.block_size)
            && new_perms[i].provenance() == prov
    by {
        if i == block_idx {
            // The inserted permission covers this block.
            assert(new_perms.dom().contains(i));
            assert(new_perms[i] == block_perm);
        } else {
            // i was free in old_view too.
            assert(!old_view.is_allocated(i));
            assert(perms.dom().contains(i));
            assert(new_perms.dom().contains(i));
            assert(new_perms[i] == perms[i]);
        }
    }
    // For allocated blocks in new_view:
    assert forall|i: int| #![trigger new_perms.dom().contains(i)]
        (0 <= i < new_view.num_data_blocks && new_view.is_allocated(i))
        implies !new_perms.dom().contains(i)
    by {
        assert(i != block_idx);
        assert(old_view.is_allocated(i));
        assert(!perms.dom().contains(i));
        assert(!new_perms.dom().contains(i));
    }
}

/// Lemma: For a freshly initialized slab, split_into_blocks produces a well-formed permission map.
proof fn lemma_fresh_slab_perms_wf(
    view: SlabView,
    perms: Map<int, PointsToRaw>,
    prov: Provenance,
)
    requires
        view.block_size > 0,
        view.num_data_blocks > 0,
        view.is_freshly_initialized(),
        forall|i: int| 0 <= i < view.num_data_blocks <==> perms.dom().contains(i),
        forall|i: int| #![trigger perms[i]]
            0 <= i < view.num_data_blocks ==>
                perms[i].is_range(view.block_addr(i), view.block_size)
                && perms[i].provenance() == prov,
    ensures
        view.perms_wf(perms, prov),
{
    // All blocks are free in a freshly initialized slab.
    // So perms_wf reduces to: every block has a permission, and no block is allocated.
    assert forall|i: int| #![trigger perms.dom().contains(i)]
        (0 <= i < view.num_data_blocks && !view.is_allocated(i))
        implies perms.dom().contains(i)
            && (#[trigger] perms[i]).is_range(view.block_addr(i), view.block_size)
            && perms[i].provenance() == prov
    by {
        // i is free (all blocks are free in fresh slab).
        assert(!view.is_allocated(i));
    }
    assert forall|i: int| #![trigger perms.dom().contains(i)]
        (0 <= i < view.num_data_blocks && view.is_allocated(i))
        implies !perms.dom().contains(i)
    by {
        // No block is allocated in a fresh slab.
        assert(view.allocated_blocks =~= Set::<int>::empty());
        assert(!view.is_allocated(i));
    }
}

} // verus!
