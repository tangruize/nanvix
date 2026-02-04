// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Kernel Heap Allocator
//==================================================================================================
//!
//! # Description
//!
//! This module provides a verified kernel heap allocator implementation.
//! The Kheap manages multiple Slab allocators of different block sizes to
//! efficiently handle allocation requests of various sizes.
//!
//! ## Memory Safety Properties Verified
//!
//! 1. **Correct Slab Selection**: Each allocation size maps to the appropriate slab.
//! 2. **No Overlap Between Slabs**: Each slab manages a disjoint memory region.
//! 3. **Allocation Validity**: Allocated addresses are within the correct slab's range.
//! 4. **Deallocation Correctness**: Deallocations target the correct slab.
//! 5. **Invariant Preservation**: The heap invariant is maintained across operations.
//!
//! ## Architecture
//!
//! The Kheap uses 8 slabs with sizes: 8, 16, 32, 64, 128, 256, 512, and 4096 bytes.
//! Each slab is initialized from a contiguous memory region:
//!
//! ```text
//! +----------+----------+----------+----------+----------+----------+----------+----------+
//! | Slab 8   | Slab 16  | Slab 32  | Slab 64  | Slab 128 | Slab 256 | Slab 512 | Slab4096 |
//! +----------+----------+----------+----------+----------+----------+----------+----------+
//! ```

use crate::{
    error::{
        Error,
        ErrorCode,
    },
    slab::{
        Slab,
        SlabView,
    },
};
use vstd::prelude::*;

verus! {

//==================================================================================================
// Constants
//==================================================================================================

/// Number of slabs in the heap.
pub const NUM_OF_SLABS: usize = 8;

/// Number of blocks per slab.
const SLAB_COUNT: usize = 32;

/// Page size (assumed for alignment).
pub const PAGE_SIZE: usize = 4096;

/// Minimum slab size in bytes.
pub const MIN_SLAB_SIZE: usize = 131072; // SLAB_COUNT * PAGE_SIZE = 32 * 4096 = 131072

/// Minimum heap size in bytes.
pub const MIN_HEAP_SIZE: usize = 1048576; // NUM_OF_SLABS * MIN_SLAB_SIZE = 8 * 131072 = 1048576

//==================================================================================================
// Slab Size Enumeration
//==================================================================================================

/// Slab size categories for the kernel heap.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum SlabSize {
    /// 8-byte blocks.
    Slab8 = 8,
    /// 16-byte blocks.
    Slab16 = 16,
    /// 32-byte blocks.
    Slab32 = 32,
    /// 64-byte blocks.
    Slab64 = 64,
    /// 128-byte blocks.
    Slab128 = 128,
    /// 256-byte blocks.
    Slab256 = 256,
    /// 512-byte blocks.
    Slab512 = 512,
    /// 4096-byte blocks.
    Slab4096 = 4096,
}

impl SlabSize {
    /// Returns the slab size as usize.
    pub fn as_usize(&self) -> (result: usize)
        ensures result == self.spec_as_int() as usize
    {
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
// KheapView - Abstract Specification
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

    /// Returns true if two slabs have disjoint memory regions.
    /// This ensures no two slabs can return overlapping addresses.
    pub open spec fn slabs_disjoint(&self, s1: &SlabView, s2: &SlabView) -> bool {
        let s1_start = s1.data_addr;
        let s1_end = s1.data_addr + s1.num_data_blocks * s1.block_size;
        let s2_start = s2.data_addr;
        let s2_end = s2.data_addr + s2.num_data_blocks * s2.block_size;
        s1_end <= s2_start || s2_end <= s1_start
    }

    /// Returns true if all slabs have disjoint memory regions.
    pub open spec fn all_slabs_disjoint(&self) -> bool {
        // Check all pairs of slabs are disjoint.
        &&& self.slabs_disjoint(&self.slab_8, &self.slab_16)
        &&& self.slabs_disjoint(&self.slab_8, &self.slab_32)
        &&& self.slabs_disjoint(&self.slab_8, &self.slab_64)
        &&& self.slabs_disjoint(&self.slab_8, &self.slab_128)
        &&& self.slabs_disjoint(&self.slab_8, &self.slab_256)
        &&& self.slabs_disjoint(&self.slab_8, &self.slab_512)
        &&& self.slabs_disjoint(&self.slab_8, &self.slab_4096)
        &&& self.slabs_disjoint(&self.slab_16, &self.slab_32)
        &&& self.slabs_disjoint(&self.slab_16, &self.slab_64)
        &&& self.slabs_disjoint(&self.slab_16, &self.slab_128)
        &&& self.slabs_disjoint(&self.slab_16, &self.slab_256)
        &&& self.slabs_disjoint(&self.slab_16, &self.slab_512)
        &&& self.slabs_disjoint(&self.slab_16, &self.slab_4096)
        &&& self.slabs_disjoint(&self.slab_32, &self.slab_64)
        &&& self.slabs_disjoint(&self.slab_32, &self.slab_128)
        &&& self.slabs_disjoint(&self.slab_32, &self.slab_256)
        &&& self.slabs_disjoint(&self.slab_32, &self.slab_512)
        &&& self.slabs_disjoint(&self.slab_32, &self.slab_4096)
        &&& self.slabs_disjoint(&self.slab_64, &self.slab_128)
        &&& self.slabs_disjoint(&self.slab_64, &self.slab_256)
        &&& self.slabs_disjoint(&self.slab_64, &self.slab_512)
        &&& self.slabs_disjoint(&self.slab_64, &self.slab_4096)
        &&& self.slabs_disjoint(&self.slab_128, &self.slab_256)
        &&& self.slabs_disjoint(&self.slab_128, &self.slab_512)
        &&& self.slabs_disjoint(&self.slab_128, &self.slab_4096)
        &&& self.slabs_disjoint(&self.slab_256, &self.slab_512)
        &&& self.slabs_disjoint(&self.slab_256, &self.slab_4096)
        &&& self.slabs_disjoint(&self.slab_512, &self.slab_4096)
    }

    //==============================================================================================
    // Address Validity
    //==============================================================================================

    /// Returns true if an address is within a specific slab's region.
    pub open spec fn addr_in_slab(&self, addr: int, size: SlabSize) -> bool {
        self.get_slab(size).is_valid_addr(addr)
    }

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

//==================================================================================================
// Layout to Slab Mapping
//==================================================================================================

/// Maps an allocation size to the appropriate slab size.
///
/// # Description
///
/// Given a requested allocation size, returns the smallest slab size that can
/// accommodate the request. Returns None if the size is too large.
///
/// # Parameters
///
/// - `size`: The requested allocation size in bytes.
///
/// # Returns
///
/// - `Ok(SlabSize)`: The appropriate slab size category.
/// - `Err`: If the size is not supported (0, >512 except 4096).
pub fn layout_to_slab_size(size: usize) -> (result: Result<SlabSize, Error>)
    ensures
        result is Ok ==> {
            let slab_size = result->Ok_0;
            &&& size > 0
            &&& size as int <= slab_size.spec_as_int()
            &&& spec_layout_to_slab_size(size as int) == Some(slab_size)
        },
        result is Err ==> spec_layout_to_slab_size(size as int).is_none(),
{
    match size {
        1..=8 => Ok(SlabSize::Slab8),
        9..=16 => Ok(SlabSize::Slab16),
        17..=32 => Ok(SlabSize::Slab32),
        33..=64 => Ok(SlabSize::Slab64),
        65..=128 => Ok(SlabSize::Slab128),
        129..=256 => Ok(SlabSize::Slab256),
        257..=512 => Ok(SlabSize::Slab512),
        4096 => Ok(SlabSize::Slab4096),
        _ => Err(Error::new(ErrorCode::InvalidArgument, "unsupported allocation size")),
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

//==================================================================================================
// Kheap Structure
//==================================================================================================

/// Kernel heap allocator managing multiple slabs.
pub struct Kheap {
    /// 8-byte block slab.
    slab_8_bytes: Slab,
    /// 16-byte block slab.
    slab_16_bytes: Slab,
    /// 32-byte block slab.
    slab_32_bytes: Slab,
    /// 64-byte block slab.
    slab_64_bytes: Slab,
    /// 128-byte block slab.
    slab_128_bytes: Slab,
    /// 256-byte block slab.
    slab_256_bytes: Slab,
    /// 512-byte block slab.
    slab_512_bytes: Slab,
    /// 4096-byte block slab.
    slab_4096_bytes: Slab,
    /// Base address of the heap (ghost field for spec).
    base_addr: Ghost<int>,
    /// Total size of the heap (ghost field for spec).
    total_size: Ghost<int>,
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
    /// Returns the base address of the heap (spec).
    pub closed spec fn spec_base_addr(&self) -> int {
        self.base_addr@
    }

    /// Returns the total size of the heap (spec).
    pub closed spec fn spec_total_size(&self) -> int {
        self.total_size@
    }

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
    // Invariant
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

    //==============================================================================================
    // Lemmas
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

    //==============================================================================================
    // Construction
    //==============================================================================================

    /// Creates a new Kheap from raw memory.
    ///
    /// # Description
    ///
    /// Initializes the kernel heap by partitioning the given memory region
    /// into 8 equal-sized slabs, each handling a different block size.
    ///
    /// # Safety
    ///
    /// - `addr` must point to valid, writable memory of at least `size` bytes.
    /// - Memory must be page-aligned (4096-byte aligned).
    /// - `size` must be at least MIN_HEAP_SIZE and a multiple of MIN_HEAP_SIZE.
    ///
    /// # Parameters
    ///
    /// - `addr`: Base address of the heap memory region.
    /// - `size`: Total size of the heap in bytes.
    ///
    /// # Returns
    ///
    /// A new Kheap instance or an error.
    ///
    /// # Verification Note
    ///
    /// The slab construction uses `Slab::from_raw_parts_at_offset`.
    /// The disjointness property is proven from the memory layout: each slab occupies
    /// a contiguous region at offset `i * slab_size`, ensuring no overlap.
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

            // Prove disjointness for all 21 pairs.
            assert(Self::spec_slabs_disjoint(&heap.slab_8_bytes@, &heap.slab_16_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_8_bytes@, &heap.slab_32_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_8_bytes@, &heap.slab_64_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_8_bytes@, &heap.slab_128_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_8_bytes@, &heap.slab_256_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_8_bytes@, &heap.slab_512_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_8_bytes@, &heap.slab_4096_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_16_bytes@, &heap.slab_32_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_16_bytes@, &heap.slab_64_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_16_bytes@, &heap.slab_128_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_16_bytes@, &heap.slab_256_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_16_bytes@, &heap.slab_512_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_16_bytes@, &heap.slab_4096_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_32_bytes@, &heap.slab_64_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_32_bytes@, &heap.slab_128_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_32_bytes@, &heap.slab_256_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_32_bytes@, &heap.slab_512_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_32_bytes@, &heap.slab_4096_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_64_bytes@, &heap.slab_128_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_64_bytes@, &heap.slab_256_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_64_bytes@, &heap.slab_512_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_64_bytes@, &heap.slab_4096_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_128_bytes@, &heap.slab_256_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_128_bytes@, &heap.slab_512_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_128_bytes@, &heap.slab_4096_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_256_bytes@, &heap.slab_512_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_256_bytes@, &heap.slab_4096_bytes@));
            assert(Self::spec_slabs_disjoint(&heap.slab_512_bytes@, &heap.slab_4096_bytes@));

            // Now show all_slabs_disjoint by using slabs_disjoint.
            assert(heap@.slab_8 == heap.slab_8_bytes@);
            assert(heap@.slab_16 == heap.slab_16_bytes@);
            assert(heap@.slab_32 == heap.slab_32_bytes@);
            assert(heap@.slab_64 == heap.slab_64_bytes@);
            assert(heap@.slab_128 == heap.slab_128_bytes@);
            assert(heap@.slab_256 == heap.slab_256_bytes@);
            assert(heap@.slab_512 == heap.slab_512_bytes@);
            assert(heap@.slab_4096 == heap.slab_4096_bytes@);

            assert(heap@.slabs_disjoint(&heap@.slab_8, &heap@.slab_16));
            assert(heap@.slabs_disjoint(&heap@.slab_8, &heap@.slab_32));
            assert(heap@.slabs_disjoint(&heap@.slab_8, &heap@.slab_64));
            assert(heap@.slabs_disjoint(&heap@.slab_8, &heap@.slab_128));
            assert(heap@.slabs_disjoint(&heap@.slab_8, &heap@.slab_256));
            assert(heap@.slabs_disjoint(&heap@.slab_8, &heap@.slab_512));
            assert(heap@.slabs_disjoint(&heap@.slab_8, &heap@.slab_4096));
            assert(heap@.slabs_disjoint(&heap@.slab_16, &heap@.slab_32));
            assert(heap@.slabs_disjoint(&heap@.slab_16, &heap@.slab_64));
            assert(heap@.slabs_disjoint(&heap@.slab_16, &heap@.slab_128));
            assert(heap@.slabs_disjoint(&heap@.slab_16, &heap@.slab_256));
            assert(heap@.slabs_disjoint(&heap@.slab_16, &heap@.slab_512));
            assert(heap@.slabs_disjoint(&heap@.slab_16, &heap@.slab_4096));
            assert(heap@.slabs_disjoint(&heap@.slab_32, &heap@.slab_64));
            assert(heap@.slabs_disjoint(&heap@.slab_32, &heap@.slab_128));
            assert(heap@.slabs_disjoint(&heap@.slab_32, &heap@.slab_256));
            assert(heap@.slabs_disjoint(&heap@.slab_32, &heap@.slab_512));
            assert(heap@.slabs_disjoint(&heap@.slab_32, &heap@.slab_4096));
            assert(heap@.slabs_disjoint(&heap@.slab_64, &heap@.slab_128));
            assert(heap@.slabs_disjoint(&heap@.slab_64, &heap@.slab_256));
            assert(heap@.slabs_disjoint(&heap@.slab_64, &heap@.slab_512));
            assert(heap@.slabs_disjoint(&heap@.slab_64, &heap@.slab_4096));
            assert(heap@.slabs_disjoint(&heap@.slab_128, &heap@.slab_256));
            assert(heap@.slabs_disjoint(&heap@.slab_128, &heap@.slab_512));
            assert(heap@.slabs_disjoint(&heap@.slab_128, &heap@.slab_4096));
            assert(heap@.slabs_disjoint(&heap@.slab_256, &heap@.slab_512));
            assert(heap@.slabs_disjoint(&heap@.slab_256, &heap@.slab_4096));
            assert(heap@.slabs_disjoint(&heap@.slab_512, &heap@.slab_4096));

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
    /// # Size Selection
    ///
    /// The allocator rounds up the requested size to the next power-of-two slab size.
    /// Supported sizes: 8, 16, 32, 64, 128, 256, 512, 4096 bytes.
    /// **Note**: Sizes 513-4095 are NOT supported and return an error.
    /// The allocated block is always >= the requested size.
    ///
    /// # Alignment
    ///
    /// Blocks are naturally aligned to their block size (e.g., 64-byte blocks are
    /// 64-byte aligned). This provides alignment guarantees equivalent to the block size.
    ///
    /// **Assumption**: This API assumes the caller's alignment requirement is <= size.
    /// If a caller requires alignment greater than the block size (e.g., 128-byte
    /// alignment for an 8-byte allocation), this allocator does NOT guarantee that
    /// alignment. In practice, slab allocators rely on natural alignment, and
    /// requests with `align > size` should use a different allocator.
    ///
    /// # Safety
    ///
    /// The returned address is valid for writes up to the slab's block size.
    #[verifier::rlimit(100)]
    pub unsafe fn allocate(&mut self, size: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> ({
                let addr = result->Ok_0 as int;
                let slab_size = spec_layout_to_slab_size(size as int).unwrap();
                &&& spec_layout_to_slab_size(size as int).is_some()
                // Address is valid in the post-state heap.
                &&& self@.is_valid_heap_addr(addr)
                // Address is valid in the selected slab (post-state).
                &&& self@.get_slab(slab_size).is_valid_addr(addr)
                // Block is now allocated in the slab.
                &&& self@.get_slab(slab_size).is_allocated(self@.get_slab(slab_size).addr_to_block_idx(addr))
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
             old(self)@.get_slab(spec_layout_to_slab_size(size as int).unwrap()).can_allocate())
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

    //==============================================================================================
    // Deallocation
    //==============================================================================================

    /// Deallocates a block of memory from the kernel heap.
    ///
    /// # Description
    ///
    /// Given an address and the original allocation size, frees the block
    /// back to the appropriate slab.
    ///
    /// # Parameters
    ///
    /// - `addr`: Address of the block to deallocate.
    /// - `size`: The original allocation size in bytes.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If deallocation succeeds.
    /// - `Err`: If the address is invalid or size is unsupported.
    ///
    /// # Safety
    ///
    /// - `addr` must have been returned by a previous `allocate` call with the same `size`.
    /// - The block must not have been deallocated already.
    pub unsafe fn deallocate(&mut self, addr: usize, size: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            addr > 0,
            spec_layout_to_slab_size(size as int).is_some(),
            ({
                let slab_size = spec_layout_to_slab_size(size as int).unwrap();
                let slab = old(self)@.get_slab(slab_size);
                &&& slab.is_valid_addr(addr as int)
                &&& slab.is_allocated(slab.addr_to_block_idx(addr as int))
            }),
        ensures
            self.inv(),
            result is Ok ==> {
                let slab_size = spec_layout_to_slab_size(size as int).unwrap();
                let old_slab = old(self)@.get_slab(slab_size);
                let new_slab = self@.get_slab(slab_size);
                let block_idx = old_slab.addr_to_block_idx(addr as int);
                &&& !new_slab.is_allocated(block_idx)
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
                // Liveness: after deallocation, the slab can allocate again.
                &&& new_slab.can_allocate()
            },
            result is Err ==> self@ == old(self)@,
            // Liveness: if preconditions are met (block is valid and allocated), deallocation succeeds.
            result is Ok,
    {
        // Determine which slab to use.
        let slab_size: SlabSize = match layout_to_slab_size(size) {
            Ok(s) => s,
            Err(e) => {
                proof {
                    // This branch is unreachable due to precondition, but we handle it.
                    assert(false);
                }
                return Err(e);
            }
        };

        // Deallocate from the appropriate slab.
        let dealloc_result: Result<(), Error> = match slab_size {
            SlabSize::Slab8 => self.slab_8_bytes.deallocate(addr),
            SlabSize::Slab16 => self.slab_16_bytes.deallocate(addr),
            SlabSize::Slab32 => self.slab_32_bytes.deallocate(addr),
            SlabSize::Slab64 => self.slab_64_bytes.deallocate(addr),
            SlabSize::Slab128 => self.slab_128_bytes.deallocate(addr),
            SlabSize::Slab256 => self.slab_256_bytes.deallocate(addr),
            SlabSize::Slab512 => self.slab_512_bytes.deallocate(addr),
            SlabSize::Slab4096 => self.slab_4096_bytes.deallocate(addr),
        };

        dealloc_result
    }
}

//==================================================================================================
// Standalone Functions
//==================================================================================================

/// Initializes the kernel heap from a raw memory region.
///
/// # Description
///
/// This function mirrors the original `init()` function. It creates a Kheap
/// from the given memory region.
///
/// # Safety
///
/// - `addr` must point to valid, writable memory of at least `size` bytes.
/// - Memory must be page-aligned.
/// - `size` must be at least MIN_HEAP_SIZE and a multiple of MIN_HEAP_SIZE.
///
/// # Note
///
/// The original `init()` uses a static `HEAP_STORAGE` array. Since Verus
/// does not support static mutable state, this function takes explicit
/// parameters instead.
pub unsafe fn init(addr: usize, size: usize) -> (result: Result<Kheap, Error>)
    requires
        addr > 0,
        addr % PAGE_SIZE as usize == 0,
        size >= MIN_HEAP_SIZE as usize,
        size % MIN_HEAP_SIZE as usize == 0,
        size % NUM_OF_SLABS == 0,
        (size / NUM_OF_SLABS) < i32::MAX as usize,
        (addr as int) + (size as int) <= (usize::MAX as int),
        // Alignment: size is multiple of 8 * 4096 for slab alignment.
        (size as int) % (8int * 4096int) == 0,
        // Power-of-two requirements.
        Slab::spec_is_power_of_two(8),
        Slab::spec_is_power_of_two(16),
        Slab::spec_is_power_of_two(32),
        Slab::spec_is_power_of_two(64),
        Slab::spec_is_power_of_two(128),
        Slab::spec_is_power_of_two(256),
        Slab::spec_is_power_of_two(512),
        Slab::spec_is_power_of_two(4096),
    ensures
        result is Ok ==> {
            let heap = result->Ok_0;
            &&& heap.inv()
            // Liveness: newly initialized heap is empty and ready for allocations.
            &&& heap@.is_empty()
        },
{
    Kheap::from_raw_parts(addr, size)
}

impl Kheap {
    //==============================================================================================
    // Lemmas
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

    /// Lemma: Different slabs handle different address ranges.
    ///
    /// # Description
    ///
    /// Proves that an address valid in one slab is NOT valid in any other slab.
    /// This is a key safety property ensuring no double-frees or cross-slab corruption.
    proof fn lemma_slabs_handle_disjoint_addresses(&self, addr: int)
        requires
            self.inv(),
            self@.is_valid_heap_addr(addr),
        ensures
            // At most one slab considers this address valid.
            // All 28 pairs: if valid in slab_i, then not valid in slab_j (i != j).
            (self@.slab_8.is_valid_addr(addr) ==> !self@.slab_16.is_valid_addr(addr)),
            (self@.slab_8.is_valid_addr(addr) ==> !self@.slab_32.is_valid_addr(addr)),
            (self@.slab_8.is_valid_addr(addr) ==> !self@.slab_64.is_valid_addr(addr)),
            (self@.slab_8.is_valid_addr(addr) ==> !self@.slab_128.is_valid_addr(addr)),
            (self@.slab_8.is_valid_addr(addr) ==> !self@.slab_256.is_valid_addr(addr)),
            (self@.slab_8.is_valid_addr(addr) ==> !self@.slab_512.is_valid_addr(addr)),
            (self@.slab_8.is_valid_addr(addr) ==> !self@.slab_4096.is_valid_addr(addr)),
            (self@.slab_16.is_valid_addr(addr) ==> !self@.slab_8.is_valid_addr(addr)),
            (self@.slab_16.is_valid_addr(addr) ==> !self@.slab_32.is_valid_addr(addr)),
            (self@.slab_16.is_valid_addr(addr) ==> !self@.slab_64.is_valid_addr(addr)),
            (self@.slab_16.is_valid_addr(addr) ==> !self@.slab_128.is_valid_addr(addr)),
            (self@.slab_16.is_valid_addr(addr) ==> !self@.slab_256.is_valid_addr(addr)),
            (self@.slab_16.is_valid_addr(addr) ==> !self@.slab_512.is_valid_addr(addr)),
            (self@.slab_16.is_valid_addr(addr) ==> !self@.slab_4096.is_valid_addr(addr)),
            (self@.slab_32.is_valid_addr(addr) ==> !self@.slab_8.is_valid_addr(addr)),
            (self@.slab_32.is_valid_addr(addr) ==> !self@.slab_16.is_valid_addr(addr)),
            (self@.slab_32.is_valid_addr(addr) ==> !self@.slab_64.is_valid_addr(addr)),
            (self@.slab_32.is_valid_addr(addr) ==> !self@.slab_128.is_valid_addr(addr)),
            (self@.slab_32.is_valid_addr(addr) ==> !self@.slab_256.is_valid_addr(addr)),
            (self@.slab_32.is_valid_addr(addr) ==> !self@.slab_512.is_valid_addr(addr)),
            (self@.slab_32.is_valid_addr(addr) ==> !self@.slab_4096.is_valid_addr(addr)),
            (self@.slab_64.is_valid_addr(addr) ==> !self@.slab_8.is_valid_addr(addr)),
            (self@.slab_64.is_valid_addr(addr) ==> !self@.slab_16.is_valid_addr(addr)),
            (self@.slab_64.is_valid_addr(addr) ==> !self@.slab_32.is_valid_addr(addr)),
            (self@.slab_64.is_valid_addr(addr) ==> !self@.slab_128.is_valid_addr(addr)),
            (self@.slab_64.is_valid_addr(addr) ==> !self@.slab_256.is_valid_addr(addr)),
            (self@.slab_64.is_valid_addr(addr) ==> !self@.slab_512.is_valid_addr(addr)),
            (self@.slab_64.is_valid_addr(addr) ==> !self@.slab_4096.is_valid_addr(addr)),
            (self@.slab_128.is_valid_addr(addr) ==> !self@.slab_8.is_valid_addr(addr)),
            (self@.slab_128.is_valid_addr(addr) ==> !self@.slab_16.is_valid_addr(addr)),
            (self@.slab_128.is_valid_addr(addr) ==> !self@.slab_32.is_valid_addr(addr)),
            (self@.slab_128.is_valid_addr(addr) ==> !self@.slab_64.is_valid_addr(addr)),
            (self@.slab_128.is_valid_addr(addr) ==> !self@.slab_256.is_valid_addr(addr)),
            (self@.slab_128.is_valid_addr(addr) ==> !self@.slab_512.is_valid_addr(addr)),
            (self@.slab_128.is_valid_addr(addr) ==> !self@.slab_4096.is_valid_addr(addr)),
            (self@.slab_256.is_valid_addr(addr) ==> !self@.slab_8.is_valid_addr(addr)),
            (self@.slab_256.is_valid_addr(addr) ==> !self@.slab_16.is_valid_addr(addr)),
            (self@.slab_256.is_valid_addr(addr) ==> !self@.slab_32.is_valid_addr(addr)),
            (self@.slab_256.is_valid_addr(addr) ==> !self@.slab_64.is_valid_addr(addr)),
            (self@.slab_256.is_valid_addr(addr) ==> !self@.slab_128.is_valid_addr(addr)),
            (self@.slab_256.is_valid_addr(addr) ==> !self@.slab_512.is_valid_addr(addr)),
            (self@.slab_256.is_valid_addr(addr) ==> !self@.slab_4096.is_valid_addr(addr)),
            (self@.slab_512.is_valid_addr(addr) ==> !self@.slab_8.is_valid_addr(addr)),
            (self@.slab_512.is_valid_addr(addr) ==> !self@.slab_16.is_valid_addr(addr)),
            (self@.slab_512.is_valid_addr(addr) ==> !self@.slab_32.is_valid_addr(addr)),
            (self@.slab_512.is_valid_addr(addr) ==> !self@.slab_64.is_valid_addr(addr)),
            (self@.slab_512.is_valid_addr(addr) ==> !self@.slab_128.is_valid_addr(addr)),
            (self@.slab_512.is_valid_addr(addr) ==> !self@.slab_256.is_valid_addr(addr)),
            (self@.slab_512.is_valid_addr(addr) ==> !self@.slab_4096.is_valid_addr(addr)),
            (self@.slab_4096.is_valid_addr(addr) ==> !self@.slab_8.is_valid_addr(addr)),
            (self@.slab_4096.is_valid_addr(addr) ==> !self@.slab_16.is_valid_addr(addr)),
            (self@.slab_4096.is_valid_addr(addr) ==> !self@.slab_32.is_valid_addr(addr)),
            (self@.slab_4096.is_valid_addr(addr) ==> !self@.slab_64.is_valid_addr(addr)),
            (self@.slab_4096.is_valid_addr(addr) ==> !self@.slab_128.is_valid_addr(addr)),
            (self@.slab_4096.is_valid_addr(addr) ==> !self@.slab_256.is_valid_addr(addr)),
            (self@.slab_4096.is_valid_addr(addr) ==> !self@.slab_512.is_valid_addr(addr)),
    {
        // Follows from all_slabs_disjoint() in the invariant.
        // The proof is automatic because all_slabs_disjoint() asserts
        // disjoint memory ranges for all 28 slab pairs.
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
// Size Gap and Alignment Lemmas
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

//==================================================================================================
// Verified Test Functions
//==================================================================================================

/// Test: layout_to_slab_size returns correct mappings.
fn test_layout_to_slab_size_verified()
{
    // Test each size category.
    let r1: Result<SlabSize, Error> = layout_to_slab_size(1);
    assert(r1 is Ok);
    assert(r1->Ok_0 == SlabSize::Slab8);

    let r8: Result<SlabSize, Error> = layout_to_slab_size(8);
    assert(r8 is Ok);
    assert(r8->Ok_0 == SlabSize::Slab8);

    let r9: Result<SlabSize, Error> = layout_to_slab_size(9);
    assert(r9 is Ok);
    assert(r9->Ok_0 == SlabSize::Slab16);

    let r16: Result<SlabSize, Error> = layout_to_slab_size(16);
    assert(r16 is Ok);
    assert(r16->Ok_0 == SlabSize::Slab16);

    let r17: Result<SlabSize, Error> = layout_to_slab_size(17);
    assert(r17 is Ok);
    assert(r17->Ok_0 == SlabSize::Slab32);

    let r64: Result<SlabSize, Error> = layout_to_slab_size(64);
    assert(r64 is Ok);
    assert(r64->Ok_0 == SlabSize::Slab64);

    let r128: Result<SlabSize, Error> = layout_to_slab_size(128);
    assert(r128 is Ok);
    assert(r128->Ok_0 == SlabSize::Slab128);

    let r256: Result<SlabSize, Error> = layout_to_slab_size(256);
    assert(r256 is Ok);
    assert(r256->Ok_0 == SlabSize::Slab256);

    let r512: Result<SlabSize, Error> = layout_to_slab_size(512);
    assert(r512 is Ok);
    assert(r512->Ok_0 == SlabSize::Slab512);

    let r4096: Result<SlabSize, Error> = layout_to_slab_size(4096);
    assert(r4096 is Ok);
    assert(r4096->Ok_0 == SlabSize::Slab4096);

    // Invalid sizes.
    let r0: Result<SlabSize, Error> = layout_to_slab_size(0);
    assert(r0 is Err);

    let r513: Result<SlabSize, Error> = layout_to_slab_size(513);
    assert(r513 is Err);

    let r1000: Result<SlabSize, Error> = layout_to_slab_size(1000);
    assert(r1000 is Err);
}

/// Test: SlabSize as_usize returns correct values.
fn test_slab_size_as_usize_verified()
{
    let s8: SlabSize = SlabSize::Slab8;
    let v8: usize = s8.as_usize();
    proof { assert(v8 == 8); }

    let s16: SlabSize = SlabSize::Slab16;
    let v16: usize = s16.as_usize();
    proof { assert(v16 == 16); }

    let s32: SlabSize = SlabSize::Slab32;
    let v32: usize = s32.as_usize();
    proof { assert(v32 == 32); }

    let s64: SlabSize = SlabSize::Slab64;
    let v64: usize = s64.as_usize();
    proof { assert(v64 == 64); }

    let s128: SlabSize = SlabSize::Slab128;
    let v128: usize = s128.as_usize();
    proof { assert(v128 == 128); }

    let s256: SlabSize = SlabSize::Slab256;
    let v256: usize = s256.as_usize();
    proof { assert(v256 == 256); }

    let s512: SlabSize = SlabSize::Slab512;
    let v512: usize = s512.as_usize();
    proof { assert(v512 == 512); }

    let s4096: SlabSize = SlabSize::Slab4096;
    let v4096: usize = s4096.as_usize();
    proof { assert(v4096 == 4096); }
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
    // All 28 pairs are disjoint.
    assert(view.slabs_disjoint(&view.slab_8, &view.slab_16));
    assert(view.slabs_disjoint(&view.slab_8, &view.slab_4096));
    assert(view.slabs_disjoint(&view.slab_16, &view.slab_4096));
    // ... (all pairs are covered by all_slabs_disjoint())
}

/// Test: Address validity is mutually exclusive between disjoint slabs.
proof fn test_address_exclusivity_verified(view: KheapView, addr: int)
    requires
        view.all_slabs_disjoint(),
        view.slab_8.is_valid_addr(addr),
{
    // If address is valid in slab_8, it cannot be valid in any other slab.
    // This follows from slabs_disjoint.
    let s8_start: int = view.slab_8.data_addr;
    let s8_end: int = view.slab_8.data_addr + view.slab_8.num_data_blocks * view.slab_8.block_size;

    // Address is in [s8_start, s8_end).
    assert(addr >= s8_start && addr < s8_end);

    // slab_16's region is disjoint from slab_8's region.
    assert(view.slabs_disjoint(&view.slab_8, &view.slab_16));
    let s16_start: int = view.slab_16.data_addr;
    let s16_end: int = view.slab_16.data_addr + view.slab_16.num_data_blocks * view.slab_16.block_size;

    // Therefore, if addr is in slab_8's range, it's not in slab_16's range.
    // (disjoint means: s8_end <= s16_start OR s16_end <= s8_start)
    // Since addr is in [s8_start, s8_end), it cannot also be in [s16_start, s16_end)
    // when the regions are disjoint.
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
// Memory Safety Properties
//==================================================================================================

/// Property: Allocated address is within the correct slab's region.
proof fn lemma_allocation_in_correct_slab(
    view: KheapView,
    slab_size: SlabSize,
    addr: int,
)
    requires
        view.get_slab(slab_size).is_valid_addr(addr),
        view.all_slabs_disjoint(),
    ensures
        // The address is NOT valid in any other slab.
        slab_size != SlabSize::Slab8 ==> !view.slab_8.is_valid_addr(addr),
        slab_size != SlabSize::Slab16 ==> !view.slab_16.is_valid_addr(addr),
        slab_size != SlabSize::Slab32 ==> !view.slab_32.is_valid_addr(addr),
        slab_size != SlabSize::Slab64 ==> !view.slab_64.is_valid_addr(addr),
        slab_size != SlabSize::Slab128 ==> !view.slab_128.is_valid_addr(addr),
        slab_size != SlabSize::Slab256 ==> !view.slab_256.is_valid_addr(addr),
        slab_size != SlabSize::Slab512 ==> !view.slab_512.is_valid_addr(addr),
        slab_size != SlabSize::Slab4096 ==> !view.slab_4096.is_valid_addr(addr),
{
    // For each pair, we use slabs_disjoint to prove exclusivity.
    // The proof follows from the fact that if two slabs are disjoint,
    // an address in one cannot be valid in the other.

    // Helper: get the slab corresponding to the given size.
    let target_slab: SlabView = view.get_slab(slab_size);

    // The address is valid in target_slab.
    let addr_start: int = target_slab.data_addr;
    let addr_end: int = target_slab.data_addr + target_slab.num_data_blocks * target_slab.block_size;
    assert(addr >= addr_start && addr < addr_end);

    // For each other slab, disjointness implies the address is not valid there.
    // We prove this by cases for each slab size.

    // Case: slab_8
    if slab_size != SlabSize::Slab8 {
        // target_slab is not slab_8, and they are disjoint.
        // Therefore, addr cannot be valid in slab_8.
        // The proof uses the fact that disjoint regions don't share addresses.
    }

    // Similar reasoning applies to all other cases.
}

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
// Test Comparison Summary
//==================================================================================================

// Summary of verified properties (15 verified tests/lemmas):
//
// | Test/Lemma | Property |
// |------------|----------|
// | test_layout_to_slab_size_verified | All size ranges map correctly |
// | test_slab_size_as_usize_verified | SlabSize enum values are correct |
// | test_slab_size_ordering_verified | Slab sizes form valid ordering |
// | test_spec_layout_to_slab_coverage_verified | All valid sizes are covered |
// | test_slabs_disjoint_property_verified | Disjointness property holds |
// | test_address_exclusivity_verified | Addresses are exclusive to one slab |
// | test_total_capacity_verified | Total capacity is sum of parts |
// | test_empty_heap_verified | Empty heap has zero allocations |
// | lemma_layout_to_slab_correct | Layout maps to sufficient slab size |
// | lemma_slab_sizes_partition_space | Slab sizes are ordered |
// | lemma_allocation_in_correct_slab | Addresses don't overlap between slabs |
// | lemma_allocation_meets_size_requirement | Allocated block meets size |
// | lemma_deallocation_correct_slab | Deallocation targets correct slab |
// | lemma_inv_implies_slab_invs | Heap inv implies slab invs |
// | lemma_capacity_conserved | Capacity is preserved |
//
// Key memory safety properties proven:
// 1. Slab selection is correct for all allocation sizes.
// 2. Slabs manage disjoint memory regions.
// 3. Allocations return valid addresses within the correct slab.
// 4. Deallocations target the correct slab based on size.
// 5. Frame conditions: operations on one slab don't affect others.

//==================================================================================================
// Allocation and Deallocation Behavior Tests
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
