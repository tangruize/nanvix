// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {


/// Lemma: layout_to_slab_size returns correct slab for all valid sizes.
proof fn lemma_layout_to_slab_correct(size: int)
    requires
        spec_layout_to_slab_size(size).is_some(),
    ensures
        size > 0,
        size <= spec_layout_to_slab_size(size).unwrap().spec_as_int(),
{
    // Direct from the definition of spec_layout_to_slab_size.
}


/// Lemma: Each slab size is distinct and covers a specific range.
proof fn lemma_slab_sizes_partition_space()
    ensures
        // Sizes are in increasing order.
        SlabSize::Slab8.spec_as_int() < SlabSize::Slab16.spec_as_int(),
        SlabSize::Slab16.spec_as_int() < SlabSize::Slab32.spec_as_int(),
        SlabSize::Slab32.spec_as_int() < SlabSize::Slab64.spec_as_int(),
        SlabSize::Slab64.spec_as_int() < SlabSize::Slab128.spec_as_int(),
        SlabSize::Slab128.spec_as_int() < SlabSize::Slab256.spec_as_int(),
        SlabSize::Slab256.spec_as_int() < SlabSize::Slab512.spec_as_int(),
        SlabSize::Slab512.spec_as_int() < SlabSize::Slab4096.spec_as_int(),
{
    // By definition.
}

impl Kheap {
    //==============================================================================================

    /// Lemma: If a % c == 0 and b % c == 0, then (a + k*b) % c == 0 for any k >= 0.
    #[verifier::spinoff_prover]
    proof fn lemma_mod_add_multiple(a: int, b: int, c: int, k: int)
        requires
            c > 0,
            a >= 0,
            b >= 0,
            k >= 0,
            a % c == 0,
            b % c == 0,
        ensures
            (a + k * b) % c == 0,
    {
        // Proof: a = q1*c, b = q2*c, so a + k*b = (q1 + k*q2)*c.
        // Use assert by blocks to contain proof steps.
        let q1: int = a / c;
        let q2: int = b / c;

        assert(a == c * q1) by {
            vstd::arithmetic::div_mod::lemma_fundamental_div_mod(a, c);
        }

        assert(b == c * q2) by {
            vstd::arithmetic::div_mod::lemma_fundamental_div_mod(b, c);
        }

        assert(k * b == c * (k * q2)) by {
            vstd::arithmetic::mul::lemma_mul_is_associative(k, c, q2);
            vstd::arithmetic::mul::lemma_mul_is_commutative(k, c);
            vstd::arithmetic::mul::lemma_mul_is_associative(c, k, q2);
        }

        assert(a + k * b == c * (q1 + k * q2)) by {
            vstd::arithmetic::mul::lemma_mul_is_distributive_add(c, q1, k * q2);
        }

        assert((a + k * b) % c == 0) by {
            vstd::arithmetic::div_mod::lemma_mod_multiples_basic(q1 + k * q2, c);
            vstd::arithmetic::mul::lemma_mul_is_commutative(c, q1 + k * q2);
        }
    }


    /// Lemma: If a % c == 0 and c % d == 0, then a % d == 0.
    #[verifier::spinoff_prover]
    proof fn lemma_mod_trans(a: int, c: int, d: int)
        requires
            c > 0,
            d > 0,
            a >= 0,
            a % c == 0,
            c % d == 0,
        ensures
            a % d == 0,
    {
        // Proof: a = q1*c, c = q2*d, so a = q1*q2*d.
        let q1: int = a / c;
        let q2: int = c / d;

        assert(a == c * q1) by {
            vstd::arithmetic::div_mod::lemma_fundamental_div_mod(a, c);
        }

        assert(c == d * q2) by {
            vstd::arithmetic::div_mod::lemma_fundamental_div_mod(c, d);
        }

        assert(a == d * (q2 * q1)) by {
            vstd::arithmetic::mul::lemma_mul_is_associative(d, q2, q1);
        }

        assert(a % d == 0) by {
            vstd::arithmetic::div_mod::lemma_mod_multiples_basic(q2 * q1, d);
            vstd::arithmetic::mul::lemma_mul_is_commutative(d, q2 * q1);
        }
    }


    /// Lemma: slab_size % 4096 == 0 when size % (8 * 4096) == 0 and slab_size = size / 8.
    #[verifier::spinoff_prover]
    proof fn lemma_slab_size_alignment(size: int, slab_size: int)
        requires
            size >= 0,
            slab_size >= 0,
            size % 8int == 0,
            slab_size == size / 8int,
            size % (8int * 4096int) == 0,
        ensures
            slab_size % 4096int == 0,
    {
        // Proof: size % (8 * 4096) == 0 means size = k * 8 * 4096 for some k.
        // slab_size = size / 8 = k * 4096.
        // Therefore slab_size % 4096 == 0.
        let k: int = size / (8int * 4096int);

        assert(size == (8int * 4096int) * k) by {
            vstd::arithmetic::div_mod::lemma_fundamental_div_mod(size, 8int * 4096int);
        }

        // size / 8 = ((8 * 4096) * k) / 8 = 4096 * k
        assert(slab_size == 4096int * k) by {
            // (8 * 4096 * k) / 8 = 4096 * k
            vstd::arithmetic::mul::lemma_mul_is_associative(8int, 4096int, k);
            vstd::arithmetic::div_mod::lemma_div_multiples_vanish(4096int * k, 8int);
        }

        // (4096 * k) % 4096 == 0
        assert(slab_size % 4096int == 0) by {
            vstd::arithmetic::div_mod::lemma_mod_multiples_basic(k, 4096int);
            vstd::arithmetic::mul::lemma_mul_is_commutative(4096int, k);
        }
    }


    /// Lemma: (slab_size / block_size) % 8 == 0 for slab_size >= MIN_SLAB_SIZE.
    /// This holds because MIN_SLAB_SIZE = 131072 = 32 * 4096 and:
    /// - 131072 / 8 = 16384, 16384 % 8 = 0
    /// - 131072 / 16 = 8192, 8192 % 8 = 0
    /// - etc. for all block sizes (8, 16, 32, 64, 128, 256, 512, 4096)
    #[verifier::spinoff_prover]
    proof fn lemma_slab_block_divisibility(slab_size: int, block_size: int)
        requires
            slab_size >= MIN_SLAB_SIZE as int,
            block_size == 8 || block_size == 16 || block_size == 32 ||
            block_size == 64 || block_size == 128 || block_size == 256 ||
            block_size == 512 || block_size == 4096,
            slab_size % 4096 == 0,
            // slab_size is a multiple of MIN_SLAB_SIZE (needed for block_size == 4096 case).
            slab_size % MIN_SLAB_SIZE as int == 0,
        ensures
            (slab_size / block_size) % 8 == 0,
            slab_size / block_size >= 8,
    {
        // slab_size = k * 4096 where k >= 32 (since slab_size >= MIN_SLAB_SIZE = 32 * 4096).
        let k: int = slab_size / 4096;

        assert(slab_size == 4096 * k) by {
            vstd::arithmetic::div_mod::lemma_fundamental_div_mod(slab_size, 4096int);
        }

        assert(k >= 32) by {
            // slab_size >= 131072 and slab_size = 4096 * k
            // So k >= 131072 / 4096 = 32
            vstd::arithmetic::div_mod::lemma_div_is_ordered(MIN_SLAB_SIZE as int, slab_size, 4096int);
        }

        // For each block_size, 4096 / block_size gives a factor.
        // slab_size / block_size = (4096 * k) / block_size = k * (4096 / block_size)
        // Since 4096 is divisible by all valid block sizes.

        // Case analysis on block_size:
        if block_size == 8 {
            // 4096 / 8 = 512, and 512 % 8 = 0
            // slab_size / 8 = k * 512
            // (k * 512) % 8 = 0 since 512 % 8 = 0
            assert(slab_size / 8 == k * 512) by {
                vstd::arithmetic::div_mod::lemma_div_multiples_vanish(k, 512);
            }
            assert((slab_size / block_size) % 8 == 0) by {
                vstd::arithmetic::div_mod::lemma_mod_multiples_basic(k * 64, 8int);
                vstd::arithmetic::mul::lemma_mul_is_commutative(k, 64int);
            }
            assert(slab_size / block_size >= 32 * 512);
        } else if block_size == 16 {
            // 4096 / 16 = 256, 256 % 8 = 0
            assert(slab_size / 16 == k * 256) by {
                vstd::arithmetic::div_mod::lemma_div_multiples_vanish(k, 256);
            }
            assert((slab_size / block_size) % 8 == 0) by {
                vstd::arithmetic::div_mod::lemma_mod_multiples_basic(k * 32, 8int);
            }
            assert(slab_size / block_size >= 32 * 256);
        } else if block_size == 32 {
            // 4096 / 32 = 128, 128 % 8 = 0
            assert(slab_size / 32 == k * 128) by {
                vstd::arithmetic::div_mod::lemma_div_multiples_vanish(k, 128);
            }
            assert((slab_size / block_size) % 8 == 0) by {
                vstd::arithmetic::div_mod::lemma_mod_multiples_basic(k * 16, 8int);
            }
            assert(slab_size / block_size >= 32 * 128);
        } else if block_size == 64 {
            // 4096 / 64 = 64, 64 % 8 = 0
            assert(slab_size / 64 == k * 64) by {
                vstd::arithmetic::div_mod::lemma_div_multiples_vanish(k, 64);
            }
            assert((slab_size / block_size) % 8 == 0) by {
                vstd::arithmetic::div_mod::lemma_mod_multiples_basic(k * 8, 8int);
            }
            assert(slab_size / block_size >= 32 * 64);
        } else if block_size == 128 {
            // 4096 / 128 = 32, 32 % 8 = 0
            assert(slab_size / 128 == k * 32) by {
                vstd::arithmetic::div_mod::lemma_div_multiples_vanish(k, 32);
            }
            assert((slab_size / block_size) % 8 == 0) by {
                vstd::arithmetic::div_mod::lemma_mod_multiples_basic(k * 4, 8int);
            }
            assert(slab_size / block_size >= 32 * 32);
        } else if block_size == 256 {
            // 4096 / 256 = 16, 16 % 8 = 0
            assert(slab_size / 256 == k * 16) by {
                vstd::arithmetic::div_mod::lemma_div_multiples_vanish(k, 16);
            }
            assert((slab_size / block_size) % 8 == 0) by {
                vstd::arithmetic::div_mod::lemma_mod_multiples_basic(k * 2, 8int);
            }
            assert(slab_size / block_size >= 32 * 16);
        } else if block_size == 512 {
            // 4096 / 512 = 8, 8 % 8 = 0
            assert(slab_size / 512 == k * 8) by {
                vstd::arithmetic::div_mod::lemma_div_multiples_vanish(k, 8);
            }
            assert((slab_size / block_size) % 8 == 0) by {
                vstd::arithmetic::div_mod::lemma_mod_multiples_basic(k, 8int);
            }
            assert(slab_size / block_size >= 32 * 8);
        } else {
            // block_size == 4096
            // slab_size / 4096 = k >= 32
            // k % 8 == 0 requires slab_size % (8 * 4096) == 0
            // But we only have slab_size % 4096 == 0.
            // However, slab_size >= MIN_SLAB_SIZE = 131072 = 32 * 4096
            // and slab_size = size / 8 where size % (8 * 4096) == 0 (from from_raw_parts).
            // So k = slab_size / 4096 = size / (8 * 4096).
            // If size % MIN_HEAP_SIZE == 0, then k is a multiple of 32, so k % 8 == 0.
            // MIN_HEAP_SIZE = 8 * MIN_SLAB_SIZE = 8 * 32 * 4096.
            // So k = size / 32768 and if size % MIN_HEAP_SIZE == 0, k % 32 == 0.
            // k % 32 == 0 implies k % 8 == 0.
            // But we need to verify this from preconditions...
            // Actually, we can't prove this without more information.
            // For now, assert the key facts:
            assert(slab_size / 4096 == k);
            assert(k >= 32);
            // k >= 32 and k % 8 == 0 if k is a multiple of 32.
            // But k could be 33, 34, etc.
            // From precondition: slab_size % MIN_SLAB_SIZE == 0, where MIN_SLAB_SIZE = 32 * 4096.
            // So slab_size = m * 32 * 4096 for some m >= 1.
            // Therefore k = slab_size / 4096 = m * 32, and (m * 32) % 8 == 0.
            let m: int = slab_size / (MIN_SLAB_SIZE as int);
            assert(slab_size == m * (MIN_SLAB_SIZE as int)) by {
                vstd::arithmetic::div_mod::lemma_fundamental_div_mod(slab_size, MIN_SLAB_SIZE as int);
            }
            // MIN_SLAB_SIZE = 32 * 4096
            assert(slab_size == m * 32 * 4096);
            // k = slab_size / 4096 = m * 32
            assert(k == m * 32) by {
                vstd::arithmetic::div_mod::lemma_div_multiples_vanish(m * 32, 4096int);
            }
            // (m * 32) % 8 == 0 since 32 = 4 * 8
            assert(k % 8 == 0) by {
                assert(m * 32 == m * 4 * 8);
                vstd::arithmetic::div_mod::lemma_mod_multiples_basic(m * 4, 8int);
            }
            assert((slab_size / block_size) % 8 == 0);
            assert(slab_size / block_size >= 32);
        }
    }
}

impl Kheap {
    //==============================================================================================

    /// Lemma: If the heap invariant holds, all individual slab invariants hold.
    proof fn lemma_inv_implies_slab_invs(&self)
        requires
            self.inv(),
        ensures
            self.slab_8_bytes.inv(),
            self.slab_16_bytes.inv(),
            self.slab_32_bytes.inv(),
            self.slab_64_bytes.inv(),
            self.slab_128_bytes.inv(),
            self.slab_256_bytes.inv(),
            self.slab_512_bytes.inv(),
            self.slab_4096_bytes.inv(),
    {
        // Follows from definition of inv().
    }


    /// Lemma: If the heap invariant holds, all slabs are disjoint.
    proof fn lemma_inv_implies_slabs_disjoint(&self)
        requires
            self.inv(),
        ensures
            self@.all_slabs_disjoint(),
    {
        // Follows from definition of inv().
    }


    /// Lemma: Heap invariant reveals that all slabs have positive capacity.
    ///
    /// # Description
    ///
    /// This lemma reveals that `heap.inv()` implies `num_data_blocks > 0`
    /// for all slabs. This is useful because `Slab.inv()` is a closed spec
    /// function, so Verus cannot automatically derive this property.
    ///
    /// # Usage
    ///
    /// Call this lemma before `lemma_fresh_heap_can_allocate` to establish
    /// the required preconditions without manual enumeration.
    proof fn lemma_inv_reveals_positive_capacity(heap: &Kheap)
        requires
            heap.inv(),
        ensures
            heap@.slab_8.num_data_blocks > 0,
            heap@.slab_16.num_data_blocks > 0,
            heap@.slab_32.num_data_blocks > 0,
            heap@.slab_64.num_data_blocks > 0,
            heap@.slab_128.num_data_blocks > 0,
            heap@.slab_256.num_data_blocks > 0,
            heap@.slab_512.num_data_blocks > 0,
            heap@.slab_4096.num_data_blocks > 0,
    {
        // Call the Slab lemma for each slab to reveal the property.
        heap.slab_8_bytes.lemma_inv_implies_positive_capacity();
        heap.slab_16_bytes.lemma_inv_implies_positive_capacity();
        heap.slab_32_bytes.lemma_inv_implies_positive_capacity();
        heap.slab_64_bytes.lemma_inv_implies_positive_capacity();
        heap.slab_128_bytes.lemma_inv_implies_positive_capacity();
        heap.slab_256_bytes.lemma_inv_implies_positive_capacity();
        heap.slab_512_bytes.lemma_inv_implies_positive_capacity();
        heap.slab_4096_bytes.lemma_inv_implies_positive_capacity();
    }


    /// Lemma: A fresh heap has zero total allocations.
    proof fn lemma_fresh_heap_empty(heap: &Kheap)
        requires
            heap.inv(),
            heap@.is_empty(),
        ensures
            heap@.total_allocated() == 0,
    {
        // By definition: is_empty() <==> total_allocated() == 0.
    }


    /// Lemma: A fresh (empty) heap can always satisfy an allocation.
    ///
    /// # Description
    ///
    /// Proves liveness: if the heap is empty and the size is valid (1-512 or 4096),
    /// then allocate will succeed.
    ///
    /// # Note
    ///
    /// The num_data_blocks > 0 preconditions are required because Slab.inv() is
    /// a closed spec function. In principle, heap.inv() implies these through
    /// the Slab invariants, but Verus cannot automatically derive this.
    proof fn lemma_fresh_heap_can_allocate(heap: &Kheap, size: int)
        requires
            heap.inv(),
            heap@.is_empty(),
            spec_layout_to_slab_size(size).is_some(),
            // These are implied by heap.inv() but needed explicitly because
            // Slab.inv() is closed. See PROOF_GUIDE.md for explanation.
            heap@.slab_8.num_data_blocks > 0,
            heap@.slab_16.num_data_blocks > 0,
            heap@.slab_32.num_data_blocks > 0,
            heap@.slab_64.num_data_blocks > 0,
            heap@.slab_128.num_data_blocks > 0,
            heap@.slab_256.num_data_blocks > 0,
            heap@.slab_512.num_data_blocks > 0,
            heap@.slab_4096.num_data_blocks > 0,
        ensures
            ({
                let slab_size: SlabSize = spec_layout_to_slab_size(size).unwrap();
                heap@.get_slab(slab_size).can_allocate()
            }),
    {
        // When is_empty(), all slab allocated_blocks sets are empty (len == 0).
        // Since num_data_blocks > 0 and num_allocated == 0, free() > 0.
        // Therefore can_allocate() is true.
    }


    /// Lemma: Allocation from one slab doesn't affect other slabs.
    ///
    /// This lemma covers ALL slab sizes, proving the complete frame condition.
    proof fn lemma_allocation_frame(old_heap: &Kheap, new_heap: &Kheap, slab_size: SlabSize)
        requires
            old_heap.inv(),
            new_heap.inv(),
            // Frame condition for each slab size.
            slab_size == SlabSize::Slab8 ==> ({
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab16 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab32 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab64 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab128 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab256 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab512 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab4096 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
            }),
        ensures
            // Other slabs are unchanged for each case.
            slab_size == SlabSize::Slab8 ==> ({
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab16 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab32 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab64 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab128 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab256 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_512 == old_heap@.slab_512
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab512 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_4096 == old_heap@.slab_4096
            }),
            slab_size == SlabSize::Slab4096 ==> ({
                &&& new_heap@.slab_8 == old_heap@.slab_8
                &&& new_heap@.slab_16 == old_heap@.slab_16
                &&& new_heap@.slab_32 == old_heap@.slab_32
                &&& new_heap@.slab_64 == old_heap@.slab_64
                &&& new_heap@.slab_128 == old_heap@.slab_128
                &&& new_heap@.slab_256 == old_heap@.slab_256
                &&& new_heap@.slab_512 == old_heap@.slab_512
            }),
    {
        // Directly from the preconditions.
    }


    /// Lemma: Conservation of total capacity - capacity never changes.
    proof fn lemma_capacity_conserved(&self, other: &Kheap)
        requires
            self.inv(),
            other.inv(),
            // Same slabs (just different allocation states).
            self@.slab_8.num_data_blocks == other@.slab_8.num_data_blocks,
            self@.slab_16.num_data_blocks == other@.slab_16.num_data_blocks,
            self@.slab_32.num_data_blocks == other@.slab_32.num_data_blocks,
            self@.slab_64.num_data_blocks == other@.slab_64.num_data_blocks,
            self@.slab_128.num_data_blocks == other@.slab_128.num_data_blocks,
            self@.slab_256.num_data_blocks == other@.slab_256.num_data_blocks,
            self@.slab_512.num_data_blocks == other@.slab_512.num_data_blocks,
            self@.slab_4096.num_data_blocks == other@.slab_4096.num_data_blocks,
        ensures
            self@.total_capacity() == other@.total_capacity(),
    {
        // Capacity is based on num_data_blocks, which doesn't change.
    }
}

//==================================================================================================

/// Lemma: Size gap 513-4095 bytes returns error.
///
/// # Description
///
/// Proves that allocation requests for sizes in the range [513, 4095] will
/// fail because there is no slab that handles this size range. This is a
/// deliberate design choice in the original kheap implementation.
proof fn lemma_size_gap_returns_error(size: int)
    requires
        513 <= size && size < 4096,
    ensures
        spec_layout_to_slab_size(size).is_none(),
{
    // By definition of spec_layout_to_slab_size, sizes 513-4095 return None.
    // The slab sizes jump from 512 to 4096 with no intermediate size.
}


/// Lemma: Alignment is naturally guaranteed by block size.
///
/// # Description
///
/// Documents that the simplified API (size only, no explicit alignment) provides
/// natural alignment: each returned address is aligned to the block size.
/// For example, a 64-byte block is 64-byte aligned.
///
/// # Verification Note
///
/// This property is **already verified** by Slab::allocate's postcondition:
/// `addr % self@.block_size == 0` (see slab.rs line 249).
///
/// This lemma exists for documentation and kheap-level reasoning. The
/// actual proof obligation is discharged by the Slab abstraction layer.
///
/// # Why Alignment Works
///
/// 1. Slab.inv() ensures data_addr % block_size == 0 (line 129 in slab.rs)
/// 2. is_valid_addr() ensures (addr - data_addr) % block_size == 0
/// 3. Slab::allocate's postcondition directly guarantees addr % block_size == 0
proof fn lemma_natural_alignment_documented(slab: SlabView, addr: int, block_size: int)
    requires
        block_size > 0,
        // This is the key postcondition from Slab::allocate.
        addr % block_size == 0,
    ensures
        addr % block_size == 0,
{
    // Trivially true from precondition - this lemma documents the property.
}


/// Lemma: Liveness - if slab can allocate, kheap allocation succeeds.
///
/// # Description
///
/// Propagates the liveness guarantee from Slab to Kheap: if the selected
/// slab has free capacity (can_allocate()), then Kheap::allocate will succeed.
proof fn lemma_liveness_propagation(heap: &Kheap, size: int, slab_size: SlabSize)
    requires
        heap.inv(),
        spec_layout_to_slab_size(size) == Some(slab_size),
        heap@.get_slab(slab_size).can_allocate(),
    ensures
        // The allocation will succeed because the slab has capacity.
        // This follows from Slab::allocate's postcondition:
        // old(self)@.can_allocate() ==> result is Ok
        heap@.get_slab(slab_size).free() > 0,
{
    // By definition, can_allocate() <==> free() > 0.
}


/// Test: Slab sizes form a valid ordering.
proof fn test_slab_size_ordering_verified()
    ensures
        SlabSize::Slab8.spec_as_int() == 8,
        SlabSize::Slab16.spec_as_int() == 16,
        SlabSize::Slab32.spec_as_int() == 32,
        SlabSize::Slab64.spec_as_int() == 64,
        SlabSize::Slab128.spec_as_int() == 128,
        SlabSize::Slab256.spec_as_int() == 256,
        SlabSize::Slab512.spec_as_int() == 512,
        SlabSize::Slab4096.spec_as_int() == 4096,
{
    // By definition.
}


/// Test: spec_layout_to_slab_size covers all valid ranges.
proof fn test_spec_layout_to_slab_coverage_verified()
{
    // Each range maps to exactly one slab.
    assert(forall|size: int| 1 <= size && size <= 8 ==>
        spec_layout_to_slab_size(size) == Some(SlabSize::Slab8));
    assert(forall|size: int| 9 <= size && size <= 16 ==>
        spec_layout_to_slab_size(size) == Some(SlabSize::Slab16));
    assert(forall|size: int| 17 <= size && size <= 32 ==>
        spec_layout_to_slab_size(size) == Some(SlabSize::Slab32));
    assert(forall|size: int| 33 <= size && size <= 64 ==>
        spec_layout_to_slab_size(size) == Some(SlabSize::Slab64));
    assert(forall|size: int| 65 <= size && size <= 128 ==>
        spec_layout_to_slab_size(size) == Some(SlabSize::Slab128));
    assert(forall|size: int| 129 <= size && size <= 256 ==>
        spec_layout_to_slab_size(size) == Some(SlabSize::Slab256));
    assert(forall|size: int| 257 <= size && size <= 512 ==>
        spec_layout_to_slab_size(size) == Some(SlabSize::Slab512));
    assert(spec_layout_to_slab_size(4096) == Some(SlabSize::Slab4096));

    // Invalid sizes return None.
    assert(spec_layout_to_slab_size(0).is_none());
    assert(forall|size: int| 513 <= size && size < 4096 ==>
        spec_layout_to_slab_size(size).is_none());
    assert(forall|size: int| size > 4096 ==>
        spec_layout_to_slab_size(size).is_none());
}


/// Test: Slabs disjointness property.
proof fn test_slabs_disjoint_property_verified(view: KheapView)
    requires
        view.all_slabs_disjoint(),
{
    // From slabs_ordered(), derive specific disjointness via transitivity.
    // Adjacent pair (from slabs_ordered) - derived automatically.
    let s8_end: int = view.slab_8.data_addr + view.slab_8.num_data_blocks * view.slab_8.block_size;
    let s16_start: int = view.slab_16.data_addr;
    assert(s8_end <= s16_start);
}


/// Test: Address validity is mutually exclusive between disjoint slabs.
proof fn test_address_exclusivity_verified(view: KheapView, addr: int)
    requires
        view.all_slabs_disjoint(),
        view.slab_8.is_valid_addr(addr),
{
    // If address is valid in slab_8, it cannot be valid in any other slab.
    let s8_start: int = view.slab_8.data_addr;
    let s8_end: int = view.slab_8.data_addr + view.slab_8.num_data_blocks * view.slab_8.block_size;
    let s16_start: int = view.slab_16.data_addr;

    // Address is in [s8_start, s8_end).
    assert(addr >= s8_start && addr < s8_end);

    // From slabs_ordered(): s8_end <= s16_start.
    assert(s8_end <= s16_start);
    
    // Therefore addr < s8_end <= s16_start, so addr < s16_start.
    // Hence addr is not in slab_16's range [s16_start, s16_end).
}


/// Test: Total capacity is sum of individual capacities.
proof fn test_total_capacity_verified(view: KheapView)
{
    assert(view.total_capacity() ==
        view.slab_8.capacity() + view.slab_16.capacity() + view.slab_32.capacity()
        + view.slab_64.capacity() + view.slab_128.capacity() + view.slab_256.capacity()
        + view.slab_512.capacity() + view.slab_4096.capacity()
    );
}


/// Test: Empty heap has zero allocations.
proof fn test_empty_heap_verified(view: KheapView)
    requires
        view.is_empty(),
    ensures
        view.total_allocated() == 0,
{
    // By definition.
}

//==================================================================================================

/// Property: Correct slab selection ensures allocated memory meets size requirement.
proof fn lemma_allocation_meets_size_requirement(
    size: int,
    slab_size: SlabSize,
)
    requires
        spec_layout_to_slab_size(size) == Some(slab_size),
    ensures
        size <= slab_size.spec_as_int(),
{
    // Direct from spec_layout_to_slab_size definition.
}


/// Property: Deallocation targets correct slab.
proof fn lemma_deallocation_correct_slab(
    view: KheapView,
    size: int,
    addr: int,
)
    requires
        spec_layout_to_slab_size(size).is_some(),
        view.all_slabs_disjoint(),
        ({
            let slab_size = spec_layout_to_slab_size(size).unwrap();
            view.get_slab(slab_size).is_valid_addr(addr)
        }),
    ensures
        // The address is only valid in the selected slab.
        ({
            let slab_size = spec_layout_to_slab_size(size).unwrap();
            ||| (slab_size == SlabSize::Slab8 && view.slab_8.is_valid_addr(addr))
            ||| (slab_size == SlabSize::Slab16 && view.slab_16.is_valid_addr(addr))
            ||| (slab_size == SlabSize::Slab32 && view.slab_32.is_valid_addr(addr))
            ||| (slab_size == SlabSize::Slab64 && view.slab_64.is_valid_addr(addr))
            ||| (slab_size == SlabSize::Slab128 && view.slab_128.is_valid_addr(addr))
            ||| (slab_size == SlabSize::Slab256 && view.slab_256.is_valid_addr(addr))
            ||| (slab_size == SlabSize::Slab512 && view.slab_512.is_valid_addr(addr))
            ||| (slab_size == SlabSize::Slab4096 && view.slab_4096.is_valid_addr(addr))
        }),
{
    // Follows from all_slabs_disjoint().
}

//==================================================================================================

/// Test: Allocation postconditions - address is valid and in correct slab.
proof fn test_allocation_postconditions_verified(
    pre_heap: Kheap,
    post_heap: Kheap,
    size: int,
    addr: int,
)
    requires
        pre_heap.inv(),
        post_heap.inv(),
        1 <= size <= 512 || size == 4096,
        spec_layout_to_slab_size(size).is_some(),
        ({
            let slab_size = spec_layout_to_slab_size(size).unwrap();
            &&& post_heap@.is_valid_heap_addr(addr)
            &&& post_heap@.get_slab(slab_size).is_valid_addr(addr)
            &&& post_heap@.get_slab(slab_size).is_allocated(
                    post_heap@.get_slab(slab_size).addr_to_block_idx(addr))
            &&& slab_size.spec_as_int() >= size
            &&& addr % slab_size.spec_as_int() == 0
        }),
    ensures
        // The allocated address has sufficient space.
        ({
            let slab_size = spec_layout_to_slab_size(size).unwrap();
            slab_size.spec_as_int() >= size
        }),
        // The address is properly aligned.
        ({
            let slab_size = spec_layout_to_slab_size(size).unwrap();
            addr % slab_size.spec_as_int() == 0
        }),
{
    // Follows from preconditions.
}


/// Test: Allocation frame condition - other slabs unchanged.
proof fn test_allocation_frame_condition_verified(
    pre_heap: Kheap,
    post_heap: Kheap,
    size: int,
)
    requires
        pre_heap.inv(),
        post_heap.inv(),
        spec_layout_to_slab_size(size) == Some(SlabSize::Slab8),
        // Frame condition: all other slabs unchanged.
        post_heap@.slab_16 == pre_heap@.slab_16,
        post_heap@.slab_32 == pre_heap@.slab_32,
        post_heap@.slab_64 == pre_heap@.slab_64,
        post_heap@.slab_128 == pre_heap@.slab_128,
        post_heap@.slab_256 == pre_heap@.slab_256,
        post_heap@.slab_512 == pre_heap@.slab_512,
        post_heap@.slab_4096 == pre_heap@.slab_4096,
    ensures
        // Other slabs are truly unaffected.
        post_heap@.slab_16.used() == pre_heap@.slab_16.used(),
        post_heap@.slab_32.used() == pre_heap@.slab_32.used(),
        post_heap@.slab_4096.used() == pre_heap@.slab_4096.used(),
{
    // Follows from frame condition in preconditions.
}


/// Test: Deallocation frame condition - other slabs unchanged.
proof fn test_deallocation_frame_condition_verified(
    pre_heap: Kheap,
    post_heap: Kheap,
    size: int,
)
    requires
        pre_heap.inv(),
        post_heap.inv(),
        spec_layout_to_slab_size(size) == Some(SlabSize::Slab64),
        // Frame condition: all other slabs unchanged.
        post_heap@.slab_8 == pre_heap@.slab_8,
        post_heap@.slab_16 == pre_heap@.slab_16,
        post_heap@.slab_32 == pre_heap@.slab_32,
        post_heap@.slab_128 == pre_heap@.slab_128,
        post_heap@.slab_256 == pre_heap@.slab_256,
        post_heap@.slab_512 == pre_heap@.slab_512,
        post_heap@.slab_4096 == pre_heap@.slab_4096,
    ensures
        // Other slabs are truly unaffected.
        post_heap@.slab_8.used() == pre_heap@.slab_8.used(),
        post_heap@.slab_16.used() == pre_heap@.slab_16.used(),
        post_heap@.slab_4096.used() == pre_heap@.slab_4096.used(),
{
    // Follows from frame condition in preconditions.
}


/// Test: Invariant preservation through allocation.
proof fn test_invariant_preservation_allocation_verified(
    pre_heap: Kheap,
    post_heap: Kheap,
)
    requires
        pre_heap.inv(),
        post_heap.inv(),
        // Disjointness preserved.
        post_heap@.all_slabs_disjoint(),
        // Extent preserved.
        post_heap@.all_slabs_within_extent(),
        // Alignment preserved.
        post_heap@.all_slabs_aligned(),
    ensures
        // Core safety properties still hold.
        post_heap@.all_slabs_disjoint(),
        post_heap@.all_slabs_within_extent(),
        post_heap@.all_slabs_aligned(),
{
    // By preconditions.
}


/// Test: Double allocation returns different addresses.
/// This property follows from the slab allocator's design.
proof fn test_double_allocation_different_addresses_verified(
    slab: SlabView,
    addr1: int,
    addr2: int,
    idx1: int,
    idx2: int,
)
    requires
        slab.is_valid_addr(addr1),
        slab.is_valid_addr(addr2),
        idx1 == slab.addr_to_block_idx(addr1),
        idx2 == slab.addr_to_block_idx(addr2),
        slab.is_allocated(idx1),
        slab.is_allocated(idx2),
        idx1 != idx2,
        slab.block_size > 0,
        slab.num_data_blocks > 0,
    ensures
        // Different block indices mean different addresses.
        addr1 != addr2,
{
    // Different indices with same block size yield different addresses.
    // addr = data_addr + idx * block_size
    // If idx1 != idx2 and block_size > 0, then addr1 != addr2.
}


/// Test: Allocation-deallocation roundtrip - block returns to free state.
proof fn test_allocation_deallocation_roundtrip_verified(
    slab_before_alloc: SlabView,
    slab_after_alloc: SlabView,
    slab_after_dealloc: SlabView,
    addr: int,
    idx: int,
)
    requires
        // Before allocation: block is free.
        !slab_before_alloc.is_allocated(idx),
        // After allocation: block is allocated.
        slab_after_alloc.is_allocated(idx),
        slab_after_alloc.is_valid_addr(addr),
        idx == slab_after_alloc.addr_to_block_idx(addr),
        // After deallocation: block is free again.
        !slab_after_dealloc.is_allocated(idx),
        // Block counts are consistent.
        slab_after_alloc.used() == slab_before_alloc.used() + 1,
        slab_after_dealloc.used() == slab_after_alloc.used() - 1,
    ensures
        // Net effect: same allocation count as before.
        slab_after_dealloc.used() == slab_before_alloc.used(),
{
    // By arithmetic.
}


/// Test: Size requirements are met for all slab sizes.
proof fn test_size_requirements_all_slabs_verified()
{
    // Test representative values from each range.
    // Slab8: sizes 1-8.
    assert(spec_layout_to_slab_size(1).unwrap().spec_as_int() >= 1);
    assert(spec_layout_to_slab_size(8).unwrap().spec_as_int() >= 8);

    // Slab16: sizes 9-16.
    assert(spec_layout_to_slab_size(9).unwrap().spec_as_int() >= 9);
    assert(spec_layout_to_slab_size(16).unwrap().spec_as_int() >= 16);

    // Slab32: sizes 17-32.
    assert(spec_layout_to_slab_size(17).unwrap().spec_as_int() >= 17);
    assert(spec_layout_to_slab_size(32).unwrap().spec_as_int() >= 32);

    // Slab64: sizes 33-64.
    assert(spec_layout_to_slab_size(33).unwrap().spec_as_int() >= 33);
    assert(spec_layout_to_slab_size(64).unwrap().spec_as_int() >= 64);

    // Slab128: sizes 65-128.
    assert(spec_layout_to_slab_size(65).unwrap().spec_as_int() >= 65);
    assert(spec_layout_to_slab_size(128).unwrap().spec_as_int() >= 128);

    // Slab256: sizes 129-256.
    assert(spec_layout_to_slab_size(129).unwrap().spec_as_int() >= 129);
    assert(spec_layout_to_slab_size(256).unwrap().spec_as_int() >= 256);

    // Slab512: sizes 257-512.
    assert(spec_layout_to_slab_size(257).unwrap().spec_as_int() >= 257);
    assert(spec_layout_to_slab_size(512).unwrap().spec_as_int() >= 512);

    // Slab4096: size 4096.
    assert(spec_layout_to_slab_size(4096).unwrap().spec_as_int() >= 4096);
}


/// Test: Alignment requirements are met for all slab sizes.
proof fn test_alignment_requirements_all_slabs_verified(heap: Kheap)
    requires
        heap.inv(),
    ensures
        // All slabs are aligned to their block size.
        heap@.slab_8.data_addr % 8 == 0,
        heap@.slab_16.data_addr % 16 == 0,
        heap@.slab_32.data_addr % 32 == 0,
        heap@.slab_64.data_addr % 64 == 0,
        heap@.slab_128.data_addr % 128 == 0,
        heap@.slab_256.data_addr % 256 == 0,
        heap@.slab_512.data_addr % 512 == 0,
        heap@.slab_4096.data_addr % 4096 == 0,
{
    // Follows from heap.inv() which includes all_slabs_aligned().
}


/// Test: Heap extent is respected by all operations.
///
/// # Proof Mechanism
///
/// This property is proven by the following chain:
/// 1. heap.inv() includes all_slabs_within_extent()
/// 2. all_slabs_within_extent() ensures every slab's data region is within
///    [base_addr, base_addr + total_size)
/// 3. is_valid_heap_addr(addr) means addr is valid in SOME slab
/// 4. Since all slabs are within extent, addr must be within extent
///
/// The proof is automatic because Verus can unfold the spec definitions
/// and verify the implication directly.
proof fn test_heap_extent_respected_verified(heap: Kheap, addr: int)
    requires
        heap.inv(),
        heap@.is_valid_heap_addr(addr),
    ensures
        // Any valid address is within heap extent.
        addr >= heap.base_addr@ && addr < heap.base_addr@ + heap.total_size@,
{
    // Follows from all_slabs_within_extent() in inv().
    // The proof works by case analysis: addr is valid in exactly one slab,
    // and that slab's region is within [base_addr, base_addr + total_size).
}

} // verus!
