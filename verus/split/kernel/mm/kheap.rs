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

use crate::libs::{
    error::{
        Error,
        ErrorCode,
    },
    slab::{
        Slab,
        SlabPerms,
        SlabView,
    },
};
use vstd::prelude::*;
use vstd::arithmetic::power2::is_pow2;
use vstd::raw_ptr::PointsToRaw;

// Include specifications.
include!("kheap.spec.rs");

// Include proofs.
include!("kheap.proof.rs");

verus! {

/// Tracked proof state for kernel heap memory ownership.
/// Holds per-slab permissions. Erased at compile time.
pub tracked struct KheapPerms {
    pub perms_8: SlabPerms,
    pub perms_16: SlabPerms,
    pub perms_32: SlabPerms,
    pub perms_64: SlabPerms,
    pub perms_128: SlabPerms,
    pub perms_256: SlabPerms,
    pub perms_512: SlabPerms,
    pub perms_4096: SlabPerms,
}

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
    ///
    /// # Justification (Extra in Verus)
    ///
    /// The original code uses `SlabSize::Variant as usize` via `#[repr(usize)]`.
    /// Verus cannot reason about `repr` casts, so this explicit conversion
    /// method is needed to bridge between spec (`spec_as_int`) and exec code.
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
}

//==================================================================================================

/// Maps an allocation size to the appropriate slab size.
///
/// # Description
///
/// Given a requested allocation size, returns the smallest slab size that can
/// accommodate the request. Returns None if the size is too large.
///
/// # Equivalence Note
///
/// This function replaces the original `Kheap::layout_to_allocator(&Layout)`.
/// The match logic is identical; only the parameter type differs:
/// - Original: takes `&Layout`, uses `layout.size()` for matching.
/// - Verus: takes `usize` directly (Verus cannot model `core::alloc::Layout`).
/// The error type is also changed from `AllocError` to `Error` because
/// `AllocError` is not available in the Verus verification context.
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

//==================================================================================================

/// Kernel heap allocator managing multiple slabs.
///
/// # Equivalence Note
///
/// Compared to the original `Kheap` struct, this version adds two ghost fields
/// (`base_addr`, `total_size`) for specification purposes. Ghost fields have no
/// runtime representation and do not affect executable behavior. The original
/// struct fields (`slab_8_bytes` through `slab_4096_bytes`) are preserved.
///
/// The original source also defines `ArenaAllocator` (a ZST implementing
/// `GlobalAlloc`) and `HeapStorage` (a page-aligned static buffer). These are
/// not modeled here because Verus cannot verify `GlobalAlloc` trait
/// implementations or static mutable state (`static mut HEAP`, `static mut
/// ALLOCATOR`, `static mut HEAP_STORAGE`).
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

impl Kheap {
    //==============================================================================================

    /// Creates a new Kheap from raw memory.
    ///
    /// # Description
    ///
    /// Initializes the kernel heap by partitioning the given memory region
    /// into 8 equal-sized slabs, each handling a different block size.
    ///
    /// # Equivalence Note
    ///
    /// The original `from_raw_parts` performs three runtime validation checks
    /// (address alignment, minimum size, size multiple) before constructing
    /// slabs via `Slab::from_raw_parts` with raw pointer arithmetic
    /// (`heap_start_addr.add(i * slab_size)`). The Verus version:
    /// - Moves the runtime validation checks to preconditions (standard Verus
    ///   pattern; the checks are still enforced, just at the caller site).
    /// - Uses `Slab::from_raw_parts(addr, slab_size, i, block_size)`
    ///   instead of pointer arithmetic, because Verus cannot reason about
    ///   `*mut u8` pointer operations. Both produce slabs at address
    ///   `addr + i * slab_size` with the same block size.
    /// - Omits `info!`/`error!` logging macros (not available in Verus context).
    /// - Adds ghost fields `base_addr` and `total_size` for specification.
    /// The executable dispatch logic (compute slab_size, construct 8 slabs at
    /// consecutive offsets) is semantically identical.
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
    /// The slab construction uses `Slab::from_raw_parts`.
    /// The disjointness property is proven from the memory layout: each slab occupies
    /// a contiguous region at offset `i * slab_size`, ensuring no overlap.
    pub unsafe fn from_raw_parts(
        addr: *mut u8,
        size: usize,
        Tracked(mem): Tracked<PointsToRaw>,
    ) -> (result: Result<(Kheap, Tracked<KheapPerms>), Error>)
        requires
            addr as int > 0,
            (addr as usize) % PAGE_SIZE as usize == 0,
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
            // Memory permission covers the entire heap region.
            mem.is_range(addr as int, size as int),
        ensures
            result is Ok ==> {
                let (heap, _perms) = result->Ok_0;
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
            assert((addr as usize) % PAGE_SIZE == 0);
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

            // Prove the new preconditions for from_raw_parts.
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

        // Split the memory permission into 8 slab-sized regions.
        let tracked slab_mem_0; let tracked slab_mem_1; let tracked slab_mem_2;
        let tracked slab_mem_3; let tracked slab_mem_4; let tracked slab_mem_5;
        let tracked slab_mem_6; let tracked slab_mem_7;
        proof {
            use vstd::set_lib::set_int_range;
            let ss: int = slab_size as int;
            let base: int = addr as int;
            // Split [base, base+8*ss) into 8 consecutive [base+i*ss, base+(i+1)*ss) regions.
            let tracked (p0, rest) = mem.split(set_int_range(base, base + ss));
            let tracked (p1, rest) = rest.split(set_int_range(base + ss, base + 2 * ss));
            let tracked (p2, rest) = rest.split(set_int_range(base + 2*ss, base + 3*ss));
            let tracked (p3, rest) = rest.split(set_int_range(base + 3*ss, base + 4*ss));
            let tracked (p4, rest) = rest.split(set_int_range(base + 4*ss, base + 5*ss));
            let tracked (p5, rest) = rest.split(set_int_range(base + 5*ss, base + 6*ss));
            let tracked (p6, p7) = rest.split(set_int_range(base + 6*ss, base + 7*ss));
            slab_mem_0 = p0; slab_mem_1 = p1; slab_mem_2 = p2; slab_mem_3 = p3;
            slab_mem_4 = p4; slab_mem_5 = p5; slab_mem_6 = p6; slab_mem_7 = p7;
        }

        // Create the 8 slabs at consecutive memory regions.
        // Each slab starts at addr + i * slab_size.
        let (slab_8, Tracked(perms_8)) = Slab::from_raw_parts(addr, slab_size, 8, Tracked(slab_mem_0))?;
        let (slab_16, Tracked(perms_16)) = Slab::from_raw_parts(addr.with_addr(addr.addr() + 1 * slab_size), slab_size, 16, Tracked(slab_mem_1))?;
        let (slab_32, Tracked(perms_32)) = Slab::from_raw_parts(addr.with_addr(addr.addr() + 2 * slab_size), slab_size, 32, Tracked(slab_mem_2))?;
        let (slab_64, Tracked(perms_64)) = Slab::from_raw_parts(addr.with_addr(addr.addr() + 3 * slab_size), slab_size, 64, Tracked(slab_mem_3))?;
        let (slab_128, Tracked(perms_128)) = Slab::from_raw_parts(addr.with_addr(addr.addr() + 4 * slab_size), slab_size, 128, Tracked(slab_mem_4))?;
        let (slab_256, Tracked(perms_256)) = Slab::from_raw_parts(addr.with_addr(addr.addr() + 5 * slab_size), slab_size, 256, Tracked(slab_mem_5))?;
        let (slab_512, Tracked(perms_512)) = Slab::from_raw_parts(addr.with_addr(addr.addr() + 6 * slab_size), slab_size, 512, Tracked(slab_mem_6))?;
        let (slab_4096, Tracked(perms_4096)) = Slab::from_raw_parts(addr.with_addr(addr.addr() + 7 * slab_size), slab_size, 4096, Tracked(slab_mem_7))?;

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
            // from_raw_parts gives us forall|i| !is_allocated(i).
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

        let tracked heap_perms = KheapPerms {
            perms_8, perms_16, perms_32, perms_64,
            perms_128, perms_256, perms_512, perms_4096,
        };

        Ok((heap, Tracked(heap_perms)))
    }

    /// Allocates a block of memory from the kernel heap.
    ///
    /// # Equivalence Note
    ///
    /// The original `allocate` takes `Layout` and returns `Result<*mut u8,
    /// AllocError>`. This version takes `size: usize` and returns
    /// `Result<usize, Error>` because Verus cannot model `core::alloc::Layout`,
    /// `AllocError`, or raw pointer types. The dispatch logic is identical:
    /// determine the slab via size-to-slab mapping, then call
    /// `slab.allocate()`. The `?` operator is replaced by explicit match
    /// (Verus limitation). The `.map_err(|_| AllocError)` is removed because
    /// the Verus `Slab::allocate` already returns `Error`.
    ///
    /// The original `alloc` method (`GlobalAlloc::alloc` on `ArenaAllocator`)
    /// is a thin wrapper that accesses `static mut HEAP` and delegates to this
    /// method. It cannot be modeled in Verus because Verus does not support
    /// `GlobalAlloc` trait implementations or static mutable state.
    ///
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
    pub unsafe fn allocate(
        &mut self,
        size: usize,
        Tracked(heap_perms): Tracked<&mut KheapPerms>,
    ) -> (result: Result<(*mut u8, Tracked<PointsToRaw>), Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> ({
                let (ptr, _block_perm) = result->Ok_0;
                let addr = ptr as int;
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
        // Hide raw pointer specs to reduce quantifier instantiation.

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

        // Allocate from the appropriate slab, with per-case proof.
        // Each arm explicitly asserts block_size preservation and invariant to
        // reduce solver search space (prevents rlimit exhaustion with large enums).
        match slab_size {
            SlabSize::Slab8 => {
                let result = self.slab_8_bytes.allocate(Tracked(&mut heap_perms.perms_8));
                proof {
                    assert(slab_size == SlabSize::Slab8);
                    assert(self.slab_8_bytes@.block_size == 8);
                    assert(8int >= size as int);
                    assert(self.inv());
                }
                result
            },
            SlabSize::Slab16 => {
                let result = self.slab_16_bytes.allocate(Tracked(&mut heap_perms.perms_16));
                proof {
                    assert(slab_size == SlabSize::Slab16);
                    assert(self.slab_16_bytes@.block_size == 16);
                    assert(16int >= size as int);
                    assert(self.inv());
                }
                result
            },
            SlabSize::Slab32 => {
                let result = self.slab_32_bytes.allocate(Tracked(&mut heap_perms.perms_32));
                proof {
                    assert(slab_size == SlabSize::Slab32);
                    assert(self.slab_32_bytes@.block_size == 32);
                    assert(32int >= size as int);
                    assert(self.inv());
                }
                result
            },
            SlabSize::Slab64 => {
                let result = self.slab_64_bytes.allocate(Tracked(&mut heap_perms.perms_64));
                proof {
                    assert(slab_size == SlabSize::Slab64);
                    assert(self.slab_64_bytes@.block_size == 64);
                    assert(64int >= size as int);
                    assert(self.inv());
                }
                result
            },
            SlabSize::Slab128 => {
                let result = self.slab_128_bytes.allocate(Tracked(&mut heap_perms.perms_128));
                proof {
                    assert(slab_size == SlabSize::Slab128);
                    assert(self.slab_128_bytes@.block_size == 128);
                    assert(128int >= size as int);
                    assert(self.inv());
                }
                result
            },
            SlabSize::Slab256 => {
                let result = self.slab_256_bytes.allocate(Tracked(&mut heap_perms.perms_256));
                proof {
                    assert(slab_size == SlabSize::Slab256);
                    assert(self.slab_256_bytes@.block_size == 256);
                    assert(256int >= size as int);
                    assert(self.inv());
                }
                result
            },
            SlabSize::Slab512 => {
                let result = self.slab_512_bytes.allocate(Tracked(&mut heap_perms.perms_512));
                proof {
                    assert(slab_size == SlabSize::Slab512);
                    assert(self.slab_512_bytes@.block_size == 512);
                    assert(512int >= size as int);
                    assert(self.inv());
                }
                result
            },
            SlabSize::Slab4096 => {
                let result = self.slab_4096_bytes.allocate(Tracked(&mut heap_perms.perms_4096));
                proof {
                    assert(slab_size == SlabSize::Slab4096);
                    assert(self.slab_4096_bytes@.block_size == 4096);
                    assert(4096int >= size as int);
                    assert(self.inv());
                }
                result
            },
        }
    }

    //==============================================================================================

    /// Deallocates a block of memory from the kernel heap.
    ///
    /// # Description
    ///
    /// Given an address and the original allocation size, frees the block
    /// back to the appropriate slab.
    ///
    /// # Equivalence Note
    ///
    /// The original `deallocate` takes `(*mut u8, Layout)` and returns
    /// `Result<(), AllocError>`. This version takes `(addr: usize, size:
    /// usize)` and returns `Result<(), Error>` for the same Verus type
    /// limitations as `allocate`. The dispatch logic is identical: determine
    /// the slab via size-to-slab mapping, then call `slab.deallocate(ptr)`.
    ///
    /// The original `dealloc` method (`GlobalAlloc::dealloc` on
    /// `ArenaAllocator`) is a thin wrapper that accesses `static mut HEAP`
    /// and delegates to this method. It cannot be modeled in Verus.
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
    pub unsafe fn deallocate(
        &mut self,
        ptr: *const u8,
        size: usize,
        Tracked(block_perm): Tracked<PointsToRaw>,
        Tracked(heap_perms): Tracked<&mut KheapPerms>,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            ptr as int > 0,
            spec_layout_to_slab_size(size as int).is_some(),
            ({
                let slab_size = spec_layout_to_slab_size(size as int).unwrap();
                let slab = old(self)@.get_slab(slab_size);
                &&& slab.is_valid_addr(ptr as int)
                &&& slab.is_allocated(slab.addr_to_block_idx(ptr as int))
            }),
        ensures
            self.inv(),
            result is Ok ==> {
                let slab_size = spec_layout_to_slab_size(size as int).unwrap();
                let old_slab = old(self)@.get_slab(slab_size);
                let new_slab = self@.get_slab(slab_size);
                let block_idx = old_slab.addr_to_block_idx(ptr as int);
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
            SlabSize::Slab8 => self.slab_8_bytes.deallocate(ptr, Tracked(block_perm), Tracked(&mut heap_perms.perms_8)),
            SlabSize::Slab16 => self.slab_16_bytes.deallocate(ptr, Tracked(block_perm), Tracked(&mut heap_perms.perms_16)),
            SlabSize::Slab32 => self.slab_32_bytes.deallocate(ptr, Tracked(block_perm), Tracked(&mut heap_perms.perms_32)),
            SlabSize::Slab64 => self.slab_64_bytes.deallocate(ptr, Tracked(block_perm), Tracked(&mut heap_perms.perms_64)),
            SlabSize::Slab128 => self.slab_128_bytes.deallocate(ptr, Tracked(block_perm), Tracked(&mut heap_perms.perms_128)),
            SlabSize::Slab256 => self.slab_256_bytes.deallocate(ptr, Tracked(block_perm), Tracked(&mut heap_perms.perms_256)),
            SlabSize::Slab512 => self.slab_512_bytes.deallocate(ptr, Tracked(block_perm), Tracked(&mut heap_perms.perms_512)),
            SlabSize::Slab4096 => self.slab_4096_bytes.deallocate(ptr, Tracked(block_perm), Tracked(&mut heap_perms.perms_4096)),
        };

        dealloc_result
    }
}

//==================================================================================================

/// Initializes the kernel heap from a raw memory region.
///
/// # Description
///
/// This function mirrors the original `init()` function. It creates a Kheap
/// from the given memory region.
///
/// # Equivalence Note
///
/// The original `init()` takes no parameters, reads from `static mut
/// HEAP_STORAGE`, and stores the result in `static mut HEAP`. This version
/// takes explicit `(addr, size)` parameters and returns `Result<Kheap, Error>`
/// because Verus cannot model static mutable state. The executable logic is
/// identical: delegate to `Kheap::from_raw_parts(addr, size)`. The original
/// `info!` logging call is omitted (not available in Verus context).
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
pub unsafe fn init(
    addr: *mut u8,
    size: usize,
    Tracked(mem): Tracked<PointsToRaw>,
) -> (result: Result<(Kheap, Tracked<KheapPerms>), Error>)
    requires
        addr as int > 0,
        (addr as usize) % PAGE_SIZE as usize == 0,
        size >= MIN_HEAP_SIZE as usize,
        size % MIN_HEAP_SIZE as usize == 0,
        size % NUM_OF_SLABS == 0,
        (size / NUM_OF_SLABS) < i32::MAX as usize,
        (addr as int) + (size as int) <= (usize::MAX as int),
        // Alignment: size is multiple of 8 * 4096 for slab alignment.
        (size as int) % (8int * 4096int) == 0,
        // Power-of-two requirements.
        is_pow2(8),
        is_pow2(16),
        is_pow2(32),
        is_pow2(64),
        is_pow2(128),
        is_pow2(256),
        is_pow2(512),
        is_pow2(4096),
        // Memory permission covers the entire region.
        mem.is_range(addr as int, size as int),
    ensures
        result is Ok ==> {
            let (heap, _perms) = result->Ok_0;
            &&& heap.inv()
            // Liveness: newly initialized heap is empty and ready for allocations.
            &&& heap@.is_empty()
        },
{
    Kheap::from_raw_parts(addr, size, Tracked(mem))
}

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

} // verus!
