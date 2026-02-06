// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

impl SlabSize {

    /// Spec function for the slab size.
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

//==================================================================================================

/// Abstract view of the Kheap state.
#[verifier::ext_equal]
pub ghost struct KheapView {
    /// View of the 8-byte slab.
    pub slab_8: SlabView,
    /// View of the 16-byte slab.
    pub slab_16: SlabView,
    /// View of the 32-byte slab.
    pub slab_32: SlabView,
    /// View of the 64-byte slab.
    pub slab_64: SlabView,
    /// View of the 128-byte slab.
    pub slab_128: SlabView,
    /// View of the 256-byte slab.
    pub slab_256: SlabView,
    /// View of the 512-byte slab.
    pub slab_512: SlabView,
    /// View of the 4096-byte slab.
    pub slab_4096: SlabView,
    /// Base address of the heap memory region (ghost).
    pub base_addr: int,
    /// Total size of the heap in bytes (ghost).
    pub total_size: int,
}


impl KheapView {
    //==============================================================================================
    // Memory Layout Properties
    //==============================================================================================

    /// Returns the slab view for a given slab size category.
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

    /// Returns total allocated blocks across all slabs.
    pub open spec fn total_allocated(&self) -> int {
        self.slab_8.used() + self.slab_16.used() + self.slab_32.used() + self.slab_64.used()
            + self.slab_128.used() + self.slab_256.used() + self.slab_512.used() + self.slab_4096.used()
    }

    /// Returns total capacity across all slabs.
    pub open spec fn total_capacity(&self) -> int {
        self.slab_8.capacity() + self.slab_16.capacity() + self.slab_32.capacity() + self.slab_64.capacity()
            + self.slab_128.capacity() + self.slab_256.capacity() + self.slab_512.capacity() + self.slab_4096.capacity()
    }

    /// Returns true if the heap is fully empty.
    pub open spec fn is_empty(&self) -> bool {
        self.total_allocated() == 0
    }

    /// Returns true if a specific slab can allocate.
    ///
    /// # Note
    ///
    /// This function appears to be a simple delegate to `get_slab(size).can_allocate()`.
    /// However, it is essential for SMT term sharing optimization. Removing it causes
    /// verification rlimit to increase by ~250% due to quantifier instantiation explosion.
    pub open spec fn can_allocate_in_slab(&self, size: SlabSize) -> bool {
        self.get_slab(size).can_allocate()
    }

    /// Returns true if all slabs are within the heap extent.
    ///
    /// # Description
    ///
    /// Ensures that all slab data regions are contained within [base_addr, base_addr + total_size).
    pub open spec fn all_slabs_within_extent(&self) -> bool {
        &&& self.slab_8.data_addr >= self.base_addr
        &&& self.slab_8.data_addr + self.slab_8.num_data_blocks * self.slab_8.block_size <= self.base_addr + self.total_size
        &&& self.slab_16.data_addr >= self.base_addr
        &&& self.slab_16.data_addr + self.slab_16.num_data_blocks * self.slab_16.block_size <= self.base_addr + self.total_size
        &&& self.slab_32.data_addr >= self.base_addr
        &&& self.slab_32.data_addr + self.slab_32.num_data_blocks * self.slab_32.block_size <= self.base_addr + self.total_size
        &&& self.slab_64.data_addr >= self.base_addr
        &&& self.slab_64.data_addr + self.slab_64.num_data_blocks * self.slab_64.block_size <= self.base_addr + self.total_size
        &&& self.slab_128.data_addr >= self.base_addr
        &&& self.slab_128.data_addr + self.slab_128.num_data_blocks * self.slab_128.block_size <= self.base_addr + self.total_size
        &&& self.slab_256.data_addr >= self.base_addr
        &&& self.slab_256.data_addr + self.slab_256.num_data_blocks * self.slab_256.block_size <= self.base_addr + self.total_size
        &&& self.slab_512.data_addr >= self.base_addr
        &&& self.slab_512.data_addr + self.slab_512.num_data_blocks * self.slab_512.block_size <= self.base_addr + self.total_size
        &&& self.slab_4096.data_addr >= self.base_addr
        &&& self.slab_4096.data_addr + self.slab_4096.num_data_blocks * self.slab_4096.block_size <= self.base_addr + self.total_size
    }

    /// Returns true if all slabs are properly aligned.
    pub open spec fn all_slabs_aligned(&self) -> bool {
        &&& self.slab_8.is_aligned()
        &&& self.slab_16.is_aligned()
        &&& self.slab_32.is_aligned()
        &&& self.slab_64.is_aligned()
        &&& self.slab_128.is_aligned()
        &&& self.slab_256.is_aligned()
        &&& self.slab_512.is_aligned()
        &&& self.slab_4096.is_aligned()
    }

    //==============================================================================================
    // Slab Memory Region Disjointness
    //==============================================================================================

    /// Returns true if s1's region ends before s2's region starts.
    /// Used for ordered slab layout.
    pub open spec fn slab_precedes(&self, s1: &SlabView, s2: &SlabView) -> bool {
        s1.data_addr + s1.num_data_blocks * s1.block_size <= s2.data_addr
    }

    /// Returns true if two slabs have disjoint memory regions.
    /// This ensures no two slabs can return overlapping addresses.
    pub open spec fn slabs_disjoint(&self, s1: &SlabView, s2: &SlabView) -> bool {
        let s1_start: int = s1.data_addr;
        let s1_end: int = s1.data_addr + s1.num_data_blocks * s1.block_size;
        let s2_start: int = s2.data_addr;
        let s2_end: int = s2.data_addr + s2.num_data_blocks * s2.block_size;
        s1_end <= s2_start || s2_end <= s1_start
    }

    /// Returns true if slabs are laid out in order (8 < 16 < 32 < ... < 4096).
    /// This is a simpler invariant than checking all 28 pairs.
    pub open spec fn slabs_ordered(&self) -> bool {
        &&& self.slab_precedes(&self.slab_8, &self.slab_16)
        &&& self.slab_precedes(&self.slab_16, &self.slab_32)
        &&& self.slab_precedes(&self.slab_32, &self.slab_64)
        &&& self.slab_precedes(&self.slab_64, &self.slab_128)
        &&& self.slab_precedes(&self.slab_128, &self.slab_256)
        &&& self.slab_precedes(&self.slab_256, &self.slab_512)
        &&& self.slab_precedes(&self.slab_512, &self.slab_4096)
    }

    /// Returns true if all slabs have disjoint memory regions.
    /// Uses slabs_ordered() for efficiency - only 7 checks instead of 28.
    /// The disjointness of all pairs follows from transitivity.
    pub open spec fn all_slabs_disjoint(&self) -> bool {
        // Slabs are ordered: s8.end <= s16.start <= ... <= s4096.start.
        // This implies all pairs are disjoint.
        self.slabs_ordered()
    }

    //==============================================================================================
    // Address Validity
    //==============================================================================================

    /// Returns true if an address is valid in any slab.
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


/// Spec version of layout_to_slab_size.
pub open spec fn spec_layout_to_slab_size(size: int) -> Option<SlabSize> {
    if 1 <= size && size <= 8 {
        Some(SlabSize::Slab8)
    } else if 9 <= size && size <= 16 {
        Some(SlabSize::Slab16)
    } else if 17 <= size && size <= 32 {
        Some(SlabSize::Slab32)
    } else if 33 <= size && size <= 64 {
        Some(SlabSize::Slab64)
    } else if 65 <= size && size <= 128 {
        Some(SlabSize::Slab128)
    } else if 129 <= size && size <= 256 {
        Some(SlabSize::Slab256)
    } else if 257 <= size && size <= 512 {
        Some(SlabSize::Slab512)
    } else if size == 4096 {
        Some(SlabSize::Slab4096)
    } else {
        None
    }
}


impl View for Kheap {
    type V = KheapView;

    closed spec fn view(&self) -> KheapView {
        KheapView {
            slab_8: self.slab_8_bytes@,
            slab_16: self.slab_16_bytes@,
            slab_32: self.slab_32_bytes@,
            slab_64: self.slab_64_bytes@,
            slab_128: self.slab_128_bytes@,
            slab_256: self.slab_256_bytes@,
            slab_512: self.slab_512_bytes@,
            slab_4096: self.slab_4096_bytes@,
            base_addr: self.base_addr@,
            total_size: self.total_size@,
        }
    }
}


impl Kheap {
    /// Spec helper for disjointness checking between two slabs.
    pub open spec fn spec_slabs_disjoint(s1: &SlabView, s2: &SlabView) -> bool {
        let s1_start: int = s1.data_addr;
        let s1_end: int = s1.data_addr + s1.num_data_blocks * s1.block_size;
        let s2_start: int = s2.data_addr;
        let s2_end: int = s2.data_addr + s2.num_data_blocks * s2.block_size;
        s1_end <= s2_start || s2_end <= s1_start
    }
}

impl Kheap {
    //==============================================================================================

    /// Invariant for the Kheap.
    ///
    /// Ensures:
    /// 1. All individual slabs satisfy their invariants (including alignment).
    /// 2. Each slab has the correct block size.
    /// 3. All slabs have disjoint memory regions.
    /// 4. All slabs are within the heap extent [base_addr, base_addr + total_size).
    /// 5. All slabs are properly aligned.
    pub closed spec fn inv(&self) -> bool {
        // All slabs satisfy their invariants (includes alignment).
        &&& self.slab_8_bytes.inv()
        &&& self.slab_16_bytes.inv()
        &&& self.slab_32_bytes.inv()
        &&& self.slab_64_bytes.inv()
        &&& self.slab_128_bytes.inv()
        &&& self.slab_256_bytes.inv()
        &&& self.slab_512_bytes.inv()
        &&& self.slab_4096_bytes.inv()
        // Block sizes are correct.
        &&& self.slab_8_bytes@.block_size == 8
        &&& self.slab_16_bytes@.block_size == 16
        &&& self.slab_32_bytes@.block_size == 32
        &&& self.slab_64_bytes@.block_size == 64
        &&& self.slab_128_bytes@.block_size == 128
        &&& self.slab_256_bytes@.block_size == 256
        &&& self.slab_512_bytes@.block_size == 512
        &&& self.slab_4096_bytes@.block_size == 4096
        // All slabs have disjoint memory regions.
        &&& self@.all_slabs_disjoint()
        // All slabs are within the heap extent.
        &&& self@.all_slabs_within_extent()
        // All slabs are properly aligned.
        &&& self@.all_slabs_aligned()
        // Base address and total size are valid.
        &&& self.base_addr@ > 0
        &&& self.total_size@ > 0
    }
}

} // verus!
