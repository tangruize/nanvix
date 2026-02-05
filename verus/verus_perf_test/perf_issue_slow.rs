// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.
//
// Minimal reproducible example for Verus performance issue
// Includes simplified bitmap to trigger quantifier complexity

use vstd::prelude::*;
use vstd::set::*;
use vstd::set_lib::*;

verus! {

//==================================================================================================
// BitmapView - Ghost state for bitmap
//==================================================================================================

#[verifier::ext_equal]
pub ghost struct BitmapView {
    pub bits: Seq<bool>,
}

impl BitmapView {
    pub open spec fn number_of_bits(&self) -> int {
        self.bits.len() as int
    }

    pub open spec fn usage(&self) -> int {
        self.count_set_bits(0, self.bits.len() as int)
    }

    pub open spec fn count_set_bits(&self, start: int, end: int) -> int
        decreases end - start
    {
        if start >= end {
            0
        } else if self.bits[start] {
            1 + self.count_set_bits(start + 1, end)
        } else {
            self.count_set_bits(start + 1, end)
        }
    }

    pub open spec fn count_free(&self) -> int {
        self.number_of_bits() - self.usage()
    }

    pub open spec fn has_free_bit(&self) -> bool {
        exists|i: int| 0 <= i < self.number_of_bits() && !self.bits[i]
    }

    pub open spec fn is_bit_set(&self, index: int) -> bool {
        0 <= index < self.number_of_bits() && self.bits[index]
    }
}

//==================================================================================================
// Bitmap - Runtime bitmap allocator
//==================================================================================================

pub struct Bitmap {
    pub number_of_bits: usize,
    pub usage: usize,
    pub ghost_bits: Ghost<Seq<bool>>,
}

impl View for Bitmap {
    type V = BitmapView;
    open spec fn view(&self) -> BitmapView {
        BitmapView { bits: self.ghost_bits@ }
    }
}

impl Bitmap {
    pub open spec fn inv(&self) -> bool {
        &&& self.number_of_bits > 0
        &&& self@.bits.len() == self.number_of_bits
        &&& self.usage <= self.number_of_bits
        &&& self.usage as int == self@.usage()
    }

    pub open spec fn is_bit_set(&self, index: int) -> bool {
        self@.is_bit_set(index)
    }

    pub fn alloc(&mut self) -> (result: Result<usize, ()>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                let index = result->Ok_0 as int;
                &&& 0 <= index < self@.number_of_bits()
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& self.is_bit_set(index)
                &&& !old(self).is_bit_set(index)
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                &&& self@.usage() == old(self)@.usage() + 1
            },
            result is Err ==> self@ == old(self)@,
            old(self)@.has_free_bit() ==> result is Ok,
    {
        if self.usage >= self.number_of_bits {
            proof {
                // All bits are set, no free bit exists
                assume(!self@.has_free_bit());
            }
            return Err(());
        }

        // Find first free bit (simplified: use usage as index)
        let index: usize = self.usage;
        
        proof {
            // Prove index is valid and not set
            assume(0 <= index < self.number_of_bits);
            assume(!self@.bits[index as int]);
        }

        // Update ghost state
        self.ghost_bits = Ghost(self.ghost_bits@.update(index as int, true));
        self.usage = self.usage + 1;

        proof {
            let old_bits = old(self)@.bits;
            let new_bits = self@.bits;
            
            // Prove postconditions
            assert(self.is_bit_set(index as int));
            assert(!old(self).is_bit_set(index as int));
            
            // Frame condition
            assume(forall|i: int| 0 <= i < self@.number_of_bits() && i != index as int
                ==> self.is_bit_set(i) == old(self).is_bit_set(i));
            
            // Usage increased by 1
            assume(self@.usage() == old(self)@.usage() + 1);
        }

        Ok(index)
    }
}

//==================================================================================================
// SlabView - Ghost state for slab allocator
//==================================================================================================

#[verifier::ext_equal]
pub ghost struct SlabView {
    pub allocated_blocks: Set<int>,
    pub num_data_blocks: int,
    pub block_size: int,
    pub data_addr: int,
}

impl SlabView {
    pub open spec fn num_allocated(&self) -> int {
        self.allocated_blocks.len() as int
    }

    // NOTE: used() function REMOVED for SLOW version

    pub open spec fn capacity(&self) -> int {
        self.num_data_blocks
    }

    pub open spec fn free(&self) -> int {
        self.capacity() - self.num_allocated()  // SLOW: direct call
    }

    pub open spec fn is_allocated(&self, block_idx: int) -> bool {
        self.allocated_blocks.contains(block_idx)
    }

    pub open spec fn can_allocate(&self) -> bool {
        self.free() > 0
    }

    pub open spec fn block_addr(&self, block_idx: int) -> int {
        self.data_addr + block_idx * self.block_size
    }

    pub open spec fn addr_to_block_idx(&self, addr: int) -> int {
        (addr - self.data_addr) / self.block_size
    }

    pub open spec fn is_valid_addr(&self, addr: int) -> bool {
        &&& addr >= self.data_addr
        &&& addr < self.data_addr + self.num_data_blocks * self.block_size
        &&& (addr - self.data_addr) % self.block_size == 0
    }

    pub open spec fn allocated_blocks_in_range(&self) -> bool {
        forall|i: int|
            #![trigger self.is_allocated(i)]
            self.is_allocated(i) ==> (0 <= i < self.num_data_blocks)
    }
}

//==================================================================================================
// Slab - Runtime slab allocator with bitmap
//==================================================================================================

pub struct Slab {
    pub index: Bitmap,
    pub data_addr: usize,
    pub num_index_blocks: usize,
    pub num_data_blocks: usize,
    pub block_size: usize,
    pub ghost_state: Ghost<SlabView>,
}

impl View for Slab {
    type V = SlabView;
    open spec fn view(&self) -> SlabView {
        self.ghost_state@
    }
}

impl Slab {
    pub open spec fn inv(&self) -> bool {
        &&& self.index.inv()
        &&& self.block_size > 0
        &&& self.num_data_blocks > 0
        &&& self.num_index_blocks >= 0
        &&& self@.block_size == self.block_size as int
        &&& self@.num_data_blocks == self.num_data_blocks as int
        &&& self@.data_addr == self.data_addr as int
        &&& self@.allocated_blocks.finite()
        &&& self@.allocated_blocks_in_range()
        &&& self@.num_allocated() <= self@.num_data_blocks
        // Link bitmap to slab state
        &&& self.index@.number_of_bits() == self.num_index_blocks as int + self.num_data_blocks as int
        &&& forall|i: int| 0 <= i < self.num_index_blocks as int ==> self.index.is_bit_set(i)
        &&& forall|j: int| 0 <= j < self.num_data_blocks as int ==>
            (self@.is_allocated(j) <==> self.index.is_bit_set(self.num_index_blocks as int + j))
        // Overflow protection
        &&& (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int
        &&& (self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int
        &&& self.data_addr > 0
    }

    /// Lemma: can_allocate() implies bitmap has_free_bit()
    pub proof fn lemma_can_allocate_implies_bitmap_has_free_bit(&self)
        requires
            self.inv(),
            self@.can_allocate(),
        ensures
            self.index@.has_free_bit(),
    {
        // can_allocate() means free() > 0
        // free() = capacity() - used() = num_data_blocks - num_allocated()
        // So num_allocated() < num_data_blocks
        // This means there exists j in [0, num_data_blocks) such that !is_allocated(j)
        // Which means !index.is_bit_set(num_index_blocks + j)
        // So index has a free bit
        
        assert(self@.free() > 0);
        assert(self@.num_allocated() < self@.num_data_blocks);
        
        // There must exist an unallocated block
        assume(exists|j: int| 0 <= j < self@.num_data_blocks && !self@.is_allocated(j));
        let j = choose|j: int| 0 <= j < self@.num_data_blocks && !self@.is_allocated(j);
        
        // That block corresponds to an unset bit in the bitmap
        let bit_idx = self.num_index_blocks as int + j;
        assert(!self.index.is_bit_set(bit_idx));
        assert(0 <= bit_idx < self.index@.number_of_bits());
        assert(!self.index@.bits[bit_idx]);
        
        // Therefore has_free_bit()
        assert(self.index@.has_free_bit());
    }

    pub fn allocate(&mut self) -> (result: Result<usize, ()>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                let addr = result->Ok_0 as int;
                let block_idx = old(self)@.addr_to_block_idx(addr);
                &&& old(self)@.is_valid_addr(addr)
                &&& 0 <= block_idx < self@.num_data_blocks
                &&& !old(self)@.is_allocated(block_idx)
                &&& self@.is_allocated(block_idx)
                &&& self@.num_data_blocks == old(self)@.num_data_blocks
                &&& self@.block_size == old(self)@.block_size
                &&& self@.data_addr == old(self)@.data_addr
                &&& forall|i: int| 0 <= i < self@.num_data_blocks && i != block_idx ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
                &&& addr > 0
            },
            result is Err ==> (self@ == old(self)@ && !old(self)@.can_allocate()),
            old(self)@.can_allocate() ==> result is Ok,
    {
        let alloc_result = self.index.alloc();

        let block: usize = match alloc_result {
            Ok(b) => b,
            Err(_) => {
                proof {
                    if old(self)@.can_allocate() {
                        old(self).lemma_can_allocate_implies_bitmap_has_free_bit();
                        // Contradiction: has_free_bit() but alloc failed
                        assert(false);
                    }
                    assert(self@ == old(self)@);
                }
                return Err(());
            }
        };

        proof {
            assert(block as int >= self.num_index_blocks as int) by {
                if (block as int) < self.num_index_blocks as int {
                    assert(old(self).index.is_bit_set(block as int));
                }
            };
        }

        let block_idx: usize = block - self.num_index_blocks;

        proof {
            assert(block_idx < self.num_data_blocks);
            assert(!old(self)@.is_allocated(block_idx as int));
            
            // Overflow proof
            assert((block_idx as int) * (self.block_size as int) < (self.num_data_blocks as int) * (self.block_size as int)) by (nonlinear_arith)
                requires block_idx < self.num_data_blocks, self.block_size > 0;
            assert((self.data_addr as int) + (block_idx as int) * (self.block_size as int) <= usize::MAX as int);
        }

        let addr: usize = self.data_addr + block_idx * self.block_size;

        // Update ghost state
        self.ghost_state = Ghost(SlabView {
            allocated_blocks: old(self)@.allocated_blocks.insert(block_idx as int),
            num_data_blocks: self@.num_data_blocks,
            block_size: self@.block_size,
            data_addr: self@.data_addr,
        });

        proof {
            let old_view = old(self)@;
            let new_view = self@;

            // Prove is_valid_addr
            assert(addr as int == old_view.block_addr(block_idx as int));
            assert((addr as int - old_view.data_addr) % old_view.block_size == 0) by (nonlinear_arith)
                requires old_view.block_size > 0,
                         (addr as int - old_view.data_addr) == (block_idx as int) * old_view.block_size;
            assert(old_view.is_valid_addr(addr as int));

            // Prove addr_to_block_idx
            assert(old_view.addr_to_block_idx(addr as int) == block_idx as int) by (nonlinear_arith)
                requires old_view.block_size > 0,
                         (addr as int - old_view.data_addr) == (block_idx as int) * old_view.block_size;

            // Prove allocation
            assert(new_view.is_allocated(block_idx as int));

            // Frame condition
            assert forall|i: int| 0 <= i < new_view.num_data_blocks && i != block_idx as int
                implies new_view.is_allocated(i) == old_view.is_allocated(i) by {
                if old_view.is_allocated(i) {
                    assert(old_view.allocated_blocks.contains(i));
                    assert(new_view.allocated_blocks.contains(i));
                }
            }

            // Prove allocated_blocks_in_range
            assert(new_view.allocated_blocks_in_range()) by {
                assert forall|i: int| new_view.is_allocated(i) implies 0 <= i < new_view.num_data_blocks by {
                    if i == block_idx as int {
                        assert(0 <= block_idx < self.num_data_blocks);
                    } else {
                        assert(old_view.is_allocated(i));
                    }
                }
            }

            // Invariant: link between bitmap and slab state
            assume(forall|j: int| 0 <= j < self.num_data_blocks as int ==>
                (self@.is_allocated(j) <==> self.index.is_bit_set(self.num_index_blocks as int + j)));
            
            // Assume full invariant holds
            assume(self.inv());
        }

        Ok(addr)
    }
}

//==================================================================================================
// KheapView and Kheap
//==================================================================================================

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SlabSize { Slab8, Slab16, Slab32, Slab64, Slab128, Slab256, Slab512, Slab4096 }

impl SlabSize {
    pub open spec fn spec_as_int(&self) -> int {
        match self {
            SlabSize::Slab8 => 8,
            SlabSize::Slab16 => 16,
            SlabSize::Slab32 => 32,
            SlabSize::Slab64 => 64,
            SlabSize::Slab128 => 128,
            SlabSize::Slab256 => 256,
            SlabSize::Slab512 => 512,
            SlabSize::Slab4096 => 4096,
        }
    }
}

#[verifier::ext_equal]
pub ghost struct KheapView {
    pub slab_8: SlabView,
    pub slab_16: SlabView,
    pub slab_32: SlabView,
    pub slab_64: SlabView,
    pub slab_128: SlabView,
    pub slab_256: SlabView,
    pub slab_512: SlabView,
    pub slab_4096: SlabView,
}

impl KheapView {
    pub open spec fn get_slab(&self, size: SlabSize) -> SlabView {
        match size {
            SlabSize::Slab8 => self.slab_8,
            SlabSize::Slab16 => self.slab_16,
            SlabSize::Slab32 => self.slab_32,
            SlabSize::Slab64 => self.slab_64,
            SlabSize::Slab128 => self.slab_128,
            SlabSize::Slab256 => self.slab_256,
            SlabSize::Slab512 => self.slab_512,
            SlabSize::Slab4096 => self.slab_4096,
        }
    }

    // NOTE: Changed to num_allocated() for SLOW version
    pub open spec fn total_allocated(&self) -> int {
        self.slab_8.num_allocated() + self.slab_16.num_allocated() + self.slab_32.num_allocated() + self.slab_64.num_allocated()
            + self.slab_128.num_allocated() + self.slab_256.num_allocated() + self.slab_512.num_allocated() + self.slab_4096.num_allocated()
    }

    pub open spec fn is_empty(&self) -> bool {
        self.total_allocated() == 0
    }

    pub open spec fn is_valid_heap_addr(&self, addr: int) -> bool {
        ||| self.slab_8.is_valid_addr(addr)
        ||| self.slab_16.is_valid_addr(addr)
        ||| self.slab_32.is_valid_addr(addr)
        ||| self.slab_64.is_valid_addr(addr)
        ||| self.slab_128.is_valid_addr(addr)
        ||| self.slab_256.is_valid_addr(addr)
        ||| self.slab_512.is_valid_addr(addr)
        ||| self.slab_4096.is_valid_addr(addr)
    }
}

pub open spec fn spec_size_to_slab(size: int) -> Option<SlabSize> {
    if 1 <= size <= 8 { Some(SlabSize::Slab8) }
    else if 9 <= size <= 16 { Some(SlabSize::Slab16) }
    else if 17 <= size <= 32 { Some(SlabSize::Slab32) }
    else if 33 <= size <= 64 { Some(SlabSize::Slab64) }
    else if 65 <= size <= 128 { Some(SlabSize::Slab128) }
    else if 129 <= size <= 256 { Some(SlabSize::Slab256) }
    else if 257 <= size <= 512 { Some(SlabSize::Slab512) }
    else if 513 <= size <= 4096 { Some(SlabSize::Slab4096) }
    else { None }
}

pub fn size_to_slab(size: usize) -> (result: Result<SlabSize, ()>)
    ensures
        result is Ok ==> spec_size_to_slab(size as int) == Some(result->Ok_0),
        result is Err ==> spec_size_to_slab(size as int).is_none(),
{
    if size == 0 { Err(()) }
    else if size <= 8 { Ok(SlabSize::Slab8) }
    else if size <= 16 { Ok(SlabSize::Slab16) }
    else if size <= 32 { Ok(SlabSize::Slab32) }
    else if size <= 64 { Ok(SlabSize::Slab64) }
    else if size <= 128 { Ok(SlabSize::Slab128) }
    else if size <= 256 { Ok(SlabSize::Slab256) }
    else if size <= 512 { Ok(SlabSize::Slab512) }
    else if size <= 4096 { Ok(SlabSize::Slab4096) }
    else { Err(()) }
}

pub struct Kheap {
    pub slab_8: Slab,
    pub slab_16: Slab,
    pub slab_32: Slab,
    pub slab_64: Slab,
    pub slab_128: Slab,
    pub slab_256: Slab,
    pub slab_512: Slab,
    pub slab_4096: Slab,
}

impl View for Kheap {
    type V = KheapView;
    open spec fn view(&self) -> KheapView {
        KheapView {
            slab_8: self.slab_8@,
            slab_16: self.slab_16@,
            slab_32: self.slab_32@,
            slab_64: self.slab_64@,
            slab_128: self.slab_128@,
            slab_256: self.slab_256@,
            slab_512: self.slab_512@,
            slab_4096: self.slab_4096@,
        }
    }
}

impl Kheap {
    pub open spec fn inv(&self) -> bool {
        &&& self.slab_8.inv()
        &&& self.slab_16.inv()
        &&& self.slab_32.inv()
        &&& self.slab_64.inv()
        &&& self.slab_128.inv()
        &&& self.slab_256.inv()
        &&& self.slab_512.inv()
        &&& self.slab_4096.inv()
    }

    /// THE KEY FUNCTION: experiences slowdown when used() is removed
    pub unsafe fn allocate(&mut self, size: usize) -> (result: Result<usize, ()>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> ({
                let addr = result->Ok_0 as int;
                let s = spec_size_to_slab(size as int).unwrap();
                &&& spec_size_to_slab(size as int).is_some()
                &&& self@.is_valid_heap_addr(addr)
                &&& self@.get_slab(s).is_valid_addr(addr)
                &&& self@.get_slab(s).is_allocated(self@.get_slab(s).addr_to_block_idx(addr))
                &&& s.spec_as_int() >= size as int
                &&& (s != SlabSize::Slab8 ==> self@.slab_8 == old(self)@.slab_8)
                &&& (s != SlabSize::Slab16 ==> self@.slab_16 == old(self)@.slab_16)
                &&& (s != SlabSize::Slab32 ==> self@.slab_32 == old(self)@.slab_32)
                &&& (s != SlabSize::Slab64 ==> self@.slab_64 == old(self)@.slab_64)
                &&& (s != SlabSize::Slab128 ==> self@.slab_128 == old(self)@.slab_128)
                &&& (s != SlabSize::Slab256 ==> self@.slab_256 == old(self)@.slab_256)
                &&& (s != SlabSize::Slab512 ==> self@.slab_512 == old(self)@.slab_512)
                &&& (s != SlabSize::Slab4096 ==> self@.slab_4096 == old(self)@.slab_4096)
            }),
            (spec_size_to_slab(size as int).is_some() &&
             old(self)@.get_slab(spec_size_to_slab(size as int).unwrap()).can_allocate())
                ==> result is Ok,
            result is Err ==> self@ == old(self)@,
    {
        let s: SlabSize = match size_to_slab(size) {
            Ok(s) => s,
            Err(_) => {
                proof { assert(self@ == old(self)@); }
                return Err(());
            }
        };

        match s {
            SlabSize::Slab8 => self.slab_8.allocate(),
            SlabSize::Slab16 => self.slab_16.allocate(),
            SlabSize::Slab32 => self.slab_32.allocate(),
            SlabSize::Slab64 => self.slab_64.allocate(),
            SlabSize::Slab128 => self.slab_128.allocate(),
            SlabSize::Slab256 => self.slab_256.allocate(),
            SlabSize::Slab512 => self.slab_512.allocate(),
            SlabSize::Slab4096 => self.slab_4096.allocate(),
        }
    }
}

} // verus!
