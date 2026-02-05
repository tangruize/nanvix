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
        SlabView,
    },
};
use vstd::prelude::*;

// Include specifications.
include!("kheap.spec.rs");

// Include proofs.
include!("kheap.proof.rs");


verus! {

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
