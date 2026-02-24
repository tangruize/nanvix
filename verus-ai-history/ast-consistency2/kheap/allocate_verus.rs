    pub unsafe fn allocate(&mut self, size: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> ({
                let addr = result->Ok_0 as int;
                let slab_size = spec_layout_to_slab_size(size as int).unwrap();
                let slab_view = self@.get_slab(slab_size);
                let block_idx = slab_view.addr_to_block_idx(addr);
                &&& spec_layout_to_slab_size(size as int).is_some()
                // Address is valid in the selected slab (implies valid in heap).
                &&& slab_view.is_valid_addr(addr)
                // Block is now allocated in the slab.
                &&& slab_view.is_allocated(block_idx)
                // Key postcondition: block_size >= requested size.
                &&& slab_size.spec_as_int() >= size as int
                // Alignment: address is aligned to block size.
                &&& addr % slab_size.spec_as_int() == 0
                // Frame: base_addr and total_size are unchanged.
                &&& self@.base_addr == old(self)@.base_addr
                &&& self@.total_size == old(self)@.total_size
                // Frame: other slabs unchanged.
                &&& (slab_size != SlabSize::Slab8 ==> self@.slab_8 == old(self)@.slab_8)
                &&& (slab_size != SlabSize::Slab16 ==> self@.slab_16 == old(self)@.slab_16)
                &&& (slab_size != SlabSize::Slab32 ==> self@.slab_32 == old(self)@.slab_32)
                &&& (slab_size != SlabSize::Slab64 ==> self@.slab_64 == old(self)@.slab_64)
                &&& (slab_size != SlabSize::Slab128 ==> self@.slab_128 == old(self)@.slab_128)
                &&& (slab_size != SlabSize::Slab256 ==> self@.slab_256 == old(self)@.slab_256)
                &&& (slab_size != SlabSize::Slab512 ==> self@.slab_512 == old(self)@.slab_512)
                &&& (slab_size != SlabSize::Slab4096 ==> self@.slab_4096 == old(self)@.slab_4096)
            }),
            // Liveness: if slab can allocate, allocation succeeds.
            // This propagates the liveness guarantee from Slab::allocate.
            (spec_layout_to_slab_size(size as int).is_some() &&
             old(self)@.can_allocate_in_slab(spec_layout_to_slab_size(size as int).unwrap()))
                ==> result is Ok,
            // Frame on error.
            result is Err ==> self@ == old(self)@,
    {
        // Hide vstd arithmetic broadcast lemmas to prevent solver slowdown.
        // hide(vstd::arithmetic::div_mod::lemma_fundamental_div_mod);
        // hide(vstd::arithmetic::div_mod::lemma_mod_multiples_basic);
        // hide(vstd::arithmetic::mul::lemma_mul_is_associative);
        // hide(vstd::arithmetic::mul::lemma_mul_is_commutative);
        // hide(vstd::arithmetic::mul::lemma_mul_is_distributive_add);

        // Determine which slab to use.
        let slab_size: SlabSize = match layout_to_slab_size(size) {
            Ok(s) => s,
            Err(e) => {
                proof {
                    assert(self@ == old(self)@);
                }
                return Err(e);
            }
        };

        // Allocate from the appropriate slab.
        let alloc_result: Result<usize, Error> = match slab_size {
            SlabSize::Slab8 => self.slab_8_bytes.allocate(),
            SlabSize::Slab16 => self.slab_16_bytes.allocate(),
            SlabSize::Slab32 => self.slab_32_bytes.allocate(),
            SlabSize::Slab64 => self.slab_64_bytes.allocate(),
            SlabSize::Slab128 => self.slab_128_bytes.allocate(),
            SlabSize::Slab256 => self.slab_256_bytes.allocate(),
            SlabSize::Slab512 => self.slab_512_bytes.allocate(),
            SlabSize::Slab4096 => self.slab_4096_bytes.allocate(),
        };

        // Process the result.
        match alloc_result {
            Ok(addr_val) => {
                proof {
                    // The allocated address is valid in the selected slab.
                    let addr: int = addr_val as int;
                    match slab_size {
                        SlabSize::Slab8 => assert(self.slab_8_bytes@.is_valid_addr(addr)),
                        SlabSize::Slab16 => assert(self.slab_16_bytes@.is_valid_addr(addr)),
                        SlabSize::Slab32 => assert(self.slab_32_bytes@.is_valid_addr(addr)),
                        SlabSize::Slab64 => assert(self.slab_64_bytes@.is_valid_addr(addr)),
                        SlabSize::Slab128 => assert(self.slab_128_bytes@.is_valid_addr(addr)),
                        SlabSize::Slab256 => assert(self.slab_256_bytes@.is_valid_addr(addr)),
                        SlabSize::Slab512 => assert(self.slab_512_bytes@.is_valid_addr(addr)),
                        SlabSize::Slab4096 => assert(self.slab_4096_bytes@.is_valid_addr(addr)),
                    }
                }
                Ok(addr_val)
            }
            Err(e) => Err(e),
        }
    }
