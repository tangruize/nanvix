// Minimal reproducible example for Verus performance issue
// ~400 lines -> trying to reduce further

use vstd::prelude::*;
use vstd::set::*;

verus! {

//==================================================================================================
// BitmapView
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
        if start >= end { 0 }
        else if self.bits[start] { 1 + self.count_set_bits(start + 1, end) }
        else { self.count_set_bits(start + 1, end) }
    }

    pub open spec fn has_free_bit(&self) -> bool {
        exists|i: int| 0 <= i < self.number_of_bits() && !self.bits[i]
    }
}

//==================================================================================================
// Bitmap
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
        0 <= index < self@.number_of_bits() && self@.bits[index]
    }

    pub fn alloc(&mut self) -> (result: Result<usize, ()>)
        requires old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> self@.usage() == old(self)@.usage() + 1,
            result is Err ==> self@ == old(self)@,
            old(self)@.has_free_bit() ==> result is Ok,
    {
        if self.usage >= self.number_of_bits {
            proof { assume(!self@.has_free_bit()); }
            return Err(());
        }
        let index: usize = self.usage;
        proof {
            assume(0 <= index < self.number_of_bits);
            assume(!self@.bits[index as int]);
        }
        self.ghost_bits = Ghost(self.ghost_bits@.update(index as int, true));
        self.usage = self.usage + 1;
        proof { assume(self@.usage() == old(self)@.usage() + 1); }
        Ok(index)
    }
}

//==================================================================================================
// SlabView
//==================================================================================================

#[verifier::ext_equal]
pub ghost struct SlabView {
    pub allocated_blocks: Set<int>,
    pub num_data_blocks: int,
}

impl SlabView {
    pub open spec fn num_allocated(&self) -> int {
        self.allocated_blocks.len() as int
    }

    // KEY: This alias is critical for SMT performance
    pub open spec fn used(&self) -> int {
        self.num_allocated()
    }

    pub open spec fn free(&self) -> int {
        self.num_data_blocks - self.used()  // FAST
    }

    pub open spec fn can_allocate(&self) -> bool {
        self.free() > 0
    }
}

//==================================================================================================
// Slab
//==================================================================================================

pub struct Slab {
    pub index: Bitmap,
    pub num_index_blocks: usize,
    pub num_data_blocks: usize,
    pub ghost_state: Ghost<SlabView>,
}

impl View for Slab {
    type V = SlabView;
    open spec fn view(&self) -> SlabView { self.ghost_state@ }
}

impl Slab {
    pub open spec fn inv(&self) -> bool {
        &&& self.index.inv()
        &&& self.num_data_blocks > 0
        &&& self@.num_data_blocks == self.num_data_blocks as int
        &&& self@.allocated_blocks.finite()
        &&& self@.num_allocated() <= self@.num_data_blocks
        &&& self.index@.number_of_bits() == self.num_index_blocks as int + self.num_data_blocks as int
        &&& forall|i: int| 0 <= i < self.num_index_blocks as int ==> self.index.is_bit_set(i)
        &&& forall|j: int| 0 <= j < self.num_data_blocks as int ==>
            (self@.allocated_blocks.contains(j) <==> self.index.is_bit_set(self.num_index_blocks as int + j))
    }

    pub fn allocate(&mut self) -> (result: Result<usize, ()>)
        requires old(self).inv(),
        ensures
            self.inv(),
            result is Err ==> self@ == old(self)@,
            old(self)@.can_allocate() ==> result is Ok,
    {
        let alloc_result = self.index.alloc();
        let block: usize = match alloc_result {
            Ok(b) => b,
            Err(_) => {
                proof {
                    if old(self)@.can_allocate() {
                        assume(old(self).index@.has_free_bit());
                        assert(false);
                    }
                }
                return Err(());
            }
        };
        let block_idx: usize = block - self.num_index_blocks;
        self.ghost_state = Ghost(SlabView {
            allocated_blocks: old(self)@.allocated_blocks.insert(block_idx as int),
            num_data_blocks: self@.num_data_blocks,
        });
        proof {
            assume(forall|j: int| 0 <= j < self.num_data_blocks as int ==>
                (self@.allocated_blocks.contains(j) <==> self.index.is_bit_set(self.num_index_blocks as int + j)));
            assume(self.inv());
        }
        Ok(0)
    }
}

//==================================================================================================
// KheapView and Kheap - 8 slabs to trigger the issue
//==================================================================================================

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
    // KEY: Uses `used()` - changing to `num_allocated()` causes slowdown
    pub open spec fn total_allocated(&self) -> int {
        self.slab_8.used() + self.slab_16.used() + self.slab_32.used() + self.slab_64.used()
            + self.slab_128.used() + self.slab_256.used() + self.slab_512.used() + self.slab_4096.used()
    }

    pub open spec fn is_empty(&self) -> bool {
        self.total_allocated() == 0
    }
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
            slab_8: self.slab_8@, slab_16: self.slab_16@, slab_32: self.slab_32@, slab_64: self.slab_64@,
            slab_128: self.slab_128@, slab_256: self.slab_256@, slab_512: self.slab_512@, slab_4096: self.slab_4096@,
        }
    }
}

impl Kheap {
    pub open spec fn inv(&self) -> bool {
        &&& self.slab_8.inv() &&& self.slab_16.inv() &&& self.slab_32.inv() &&& self.slab_64.inv()
        &&& self.slab_128.inv() &&& self.slab_256.inv() &&& self.slab_512.inv() &&& self.slab_4096.inv()
    }

    // THE KEY FUNCTION
    pub fn allocate(&mut self, which: u8) -> (result: Result<(), ()>)
        requires old(self).inv(),
        ensures self.inv(),
    {
        match which {
            0 => { let _ = self.slab_8.allocate(); }
            1 => { let _ = self.slab_16.allocate(); }
            2 => { let _ = self.slab_32.allocate(); }
            3 => { let _ = self.slab_64.allocate(); }
            4 => { let _ = self.slab_128.allocate(); }
            5 => { let _ = self.slab_256.allocate(); }
            6 => { let _ = self.slab_512.allocate(); }
            _ => { let _ = self.slab_4096.allocate(); }
        }
        Ok(())
    }
}

} // verus!
