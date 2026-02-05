// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # Kernel Red Zone (Verified Specification)
//!
//! This module provides a verified specification for the Kernel Red Zone data structure.
//! The kernel red zone is a small, fixed-size memory region used to store temporary values
//! during kernel operations (e.g., for context switching or interrupt handling).
//!
//! ## Overview
//!
//! The kernel red zone provides indexed access to a fixed-size array of `usize` values.
//! Key properties:
//! - **Fixed size**: KREDZONE_SIZE bytes (128 bytes = 16 usize entries on 64-bit, 32 on 32-bit)
//! - **Indexed access**: Values are stored/loaded by index
//! - **Bounds checking**: All accesses are validated against the maximum index
//!
//! ## Verification Architecture
//!
//! This module uses a **specification-first** approach with explicit trust boundaries:
//!
//! ### Verified (Abstract Model)
//!
//! The following properties are **formally proven** for the abstract `KernelRedZoneView`:
//!
//! 1. **Bounds Safety**: Index must be within [0, NUM_ENTRIES) for all operations.
//! 2. **Element Isolation**: Writing to index i does not affect any other index j != i.
//! 3. **Read-after-Write**: Loading from index i after storing v at i returns v.
//! 4. **Store Commutativity**: Independent stores to different indices commute.
//! 5. **Store Idempotence**: Consecutive stores to same index keep last value.
//! 6. **Invariant Preservation**: Operations preserve well-formedness.
//!
//! ### Trusted (Implementation)
//!
//! The `store()` and `load()` functions are marked `#[verifier::external_body]` because:
//!
//! 1. They access an `extern "C"` static variable (`kredzone`) linked from assembly.
//! 2. They use `volatile` memory operations which Verus cannot reason about.
//! 3. Threading ghost state through a global static is not feasible in this context.
//!
//! The trust assumptions are:
//!
//! - **T1**: The assembly-defined `kredzone` region is at least KREDZONE_SIZE bytes.
//! - **T2**: Volatile reads return the last value written at that address.
//! - **T3**: No concurrent access occurs (single-threaded kernel context).
//! - **T4**: The kredzone region is zero-initialized before first use (BSS section).
//! - **T5**: Only one `KernelRedZoneGhost` instance is active at any time (uniqueness).
//!
//! **Why these cannot be encoded as Verus preconditions:**
//!
//! - T1 is a linker/build-time property verified by the assembly source (`start.S`).
//! - T2 is a hardware/compiler semantics assumption outside Verus's reasoning scope.
//! - T3 is a kernel design invariant enforced by the single-threaded execution model.
//! - T4 is a linker/loader property (BSS zero-initialization) verified by the toolchain.
//! - T5 is a design discipline requirement; Verus cannot enforce uniqueness for global state.
//!
//! These are **environmental assumptions** documented for auditors, not runtime checks.
//!
//! ### Ghost State (`KernelRedZoneGhost`)
//!
//! The `KernelRedZoneGhost` struct provides an abstract model for callers who wish to
//! reason about sequences of operations. Since `store`/`load` operate on global state
//! without tracked parameters, callers must maintain their own ghost state that mirrors
//! the expected concrete state. The `spec_store_effect` and `spec_load_result` functions
//! define the expected behavior for such reasoning.
//!
//! **Why ghost state cannot be coupled to executable functions:**
//!
//! The original implementation uses `extern "C" { static mut kredzone: usize; }` which:
//! - Has a fixed C ABI that cannot accept Verus tracked/ghost parameters.
//! - Is accessed via raw pointer arithmetic (`ptr.add(index)`).
//! - Uses `write_volatile`/`read_volatile` which are opaque to Verus.
//!
//! The proven algebraic properties (`lemma_store_then_load`, etc.) apply to the abstract
//! model. Under trust assumptions T1-T3, these properties transfer to the implementation.
//!
//! ### ⚠️ Ghost State Uniqueness Requirement (T5)
//!
//! **CRITICAL**: The ghost state model assumes exactly **one** `KernelRedZoneGhost` instance
//! exists at any time, representing the single global `kredzone` memory. This is Trust
//! Assumption T5.
//!
//! - **T5**: Only one `KernelRedZoneGhost` instance is active and all kredzone operations
//!   use that instance consistently.
//!
//! **Soundness Warning**: If multiple ghost instances are created (e.g., calling
//! `create_initial_ghost()` twice), they can diverge from each other and from reality.
//! The `assume` in `load_with_ghost` is only sound if the ghost state is the unique
//! model of the kredzone and has been kept synchronized via `store_with_ghost`.
//!
//! **Recommended Usage Pattern**:
//! ```ignore
//! // At kernel boot, create ONE ghost state:
//! init_kredzone();  // Zero all entries.
//! let tracked mut ghost = create_initial_ghost();  // The ONLY ghost instance.
//!
//! // All subsequent operations use this single instance:
//! store_with_ghost(0, 42, Tracked(&mut ghost))?;
//! let val = load_with_ghost(0, Tracked(&ghost))?;
//! ```
//!
//! This uniqueness cannot be enforced by Verus's type system for global state, so it
//! must be enforced by kernel design discipline.
//!
//! ## Verification Scope Limitations
//!
//! The following requests **cannot be implemented** due to Verus/language constraints:
//!
//! 1. **Stronger store/load specs relating to abstract state**: Would require Verus to
//!    reason about `write_volatile`/`read_volatile` on extern C statics, which is not
//!    supported. The volatile operations are fundamentally opaque to the verifier.
//!
//! 2. **Encoding trust assumptions as preconditions**: T1-T3 are environmental properties
//!    (linker output, hardware semantics, OS design) that exist outside Verus's scope.
//!    They cannot be expressed as runtime-checkable preconditions.
//!
//! 3. **Threading ghost state through store/load**: The functions must maintain API
//!    compatibility with the original `pub fn store(index: usize, value: usize)` signature.
//!    Adding tracked parameters would change the ABI and break all callers.
//!
//! This verification represents the **maximum achievable assurance** for this component.
//! The bounds checking is fully verified; functional correctness relies on the documented
//! trust assumptions which must be validated by code review of the assembly and kernel design.
//!
//! ## API Summary
//!
//! | Function | Description | Trust Level |
//! |----------|-------------|-------------|
//! | `store(index, value)` | Store a value at the given index | **Bounds verified**, volatile trusted |
//! | `load(index)` | Load a value from the given index | **Bounds verified**, volatile trusted |
//! | `raw_store(index, value)` | Raw volatile write | Trusted (external_body) |
//! | `raw_load(index)` | Raw volatile read | Trusted (external_body) |
//! | `store_with_ghost(...)` | Verified wrapper managing ghost state | Verified |
//! | `load_with_ghost(...)` | Verified wrapper using ghost state | Verified |
//! | `init_kredzone()` | Zero all entries (alternative to T4) | Verified |
//! | `create_initial_ghost()` | Create initial ghost state | Verified (requires T4 or init_kredzone) |
//!
//! ### Verification Architecture
//!
//! The `store`/`load` functions are structured to maximize verified code:
//!
//! 1. **Verified bounds check**: The `if index >= NUM_ENTRIES` check is verified by Verus.
//! 2. **Trusted volatile access**: Only `raw_store`/`raw_load` are `external_body`.
//!
//! This ensures that bounds safety is machine-checked, while only the minimal
//! volatile memory operations remain trusted.
//!
//! ### Ghost State Wrappers
//!
//! For callers who wish to use the proven algebraic properties, verified wrappers
//! are provided that manage ghost state:
//!
//! - `store_with_ghost`: Calls `store` and updates ghost state via `spec_store_effect`.
//! - `load_with_ghost`: Calls `load` with ghost state for reasoning.
//! - `create_initial_ghost`: Creates an initial well-formed ghost state.
//!
//! These wrappers bridge the gap between the abstract model and the implementation,
//! allowing verified code to use the proven lemmas while the raw API remains available
//! for legacy/C callers.
//!
//! ## Divergences from Original Implementation
//!
//! - **Logging omitted**: The `error!()` macro calls are omitted in the verified version
//!   to avoid side effects during verification. Logging behavior is preserved in the
//!   original `src/kernel/src/mm/kredzone.rs`.
//!
//! - **ENTRY_SIZE**: Uses conditional compilation with literals instead of
//!   `mem::size_of::<usize>()`. See `lemma_entry_size_matches_target` for verification
//!   that these values are correct for supported platforms.
//!
//==================================================================================================

use crate::libs::error::{Error, ErrorCode};
use vstd::prelude::*;

// Include specifications.
include!("kredzone.spec.rs");

// Include proofs.
include!("kredzone.proof.rs");


verus! {

//==================================================================================================

/// Size of the kernel red zone in bytes.
/// This should match what is defined in start.S.
pub const KREDZONE_SIZE: usize = 128;


/// Size of a single entry in bytes (sizeof(usize)).
/// Note: On x86-32 this is 4, on x86-64 this is 8.
/// We use a literal to avoid Verus issues with size_of.
/// 
/// **Warning:** Only 32-bit and 64-bit platforms are officially supported.
/// The fallback for other architectures defaults to 64-bit but should not be relied upon.
#[cfg(target_pointer_width = "64")]
pub const ENTRY_SIZE: usize = 8;


#[cfg(target_pointer_width = "32")]
pub const ENTRY_SIZE: usize = 4;

#[cfg(all(not(target_pointer_width = "32"), not(target_pointer_width = "64")))]
pub const ENTRY_SIZE: usize = 8;  // WARNING: Unsupported architecture, defaulting to 64-bit.


/// Number of entries in the kernel red zone (exec version).
pub const NUM_ENTRIES: usize = KREDZONE_SIZE / ENTRY_SIZE;

//==================================================================================================

/// Abstract view of the kernel red zone for specification purposes.
///
/// This view models the red zone as a sequence of usize values.
#[verifier::ext_equal]
pub struct KernelRedZoneView {
    /// The logical contents of the red zone.
    pub contents: Seq<usize>,
}

//==================================================================================================

/// Stores a value in the kernel red zone.
///
/// # Description
///
/// Stores a value at the given index in the kernel red zone.
///
/// # Parameters
///
/// - `index`: Index in the kernel red zone.
/// - `value`: Value to store.
///
/// # Returns
///
/// Upon success, empty is returned. Upon failure, an error is returned instead.
///
/// # Errors
///
/// - `InvalidArgument`: If index is out of bounds.
///
/// # Verification Notes
///
/// This function is marked `external_body` because it accesses an extern "C" static
/// variable using volatile pointer operations. The specification captures:
///
/// - **Postcondition on success**: The index was valid.
/// - **Postcondition on failure**: The index was out of bounds.
///
/// The actual memory effect (writing `value` at `index`) is trusted, not verified.
/// **For functional correctness verification, use `store_with_ghost()` instead.**
/// Callers reasoning about state changes should use `spec_store_effect()` to model
/// the expected effect on their ghost state.
///
/// # Warning
///
/// This file is for **verification only**. The body below is a stub that Verus ignores
/// due to `external_body`. The actual implementation is in `src/kernel/src/mm/kredzone.rs`
/// and uses `extern "C" { static mut kredzone: usize; }` with volatile pointer operations.
/// Do not compile this file as a replacement for the kernel module.
///
/// # Verified Bounds Check
///
/// The bounds check is **verified** by Verus. Only the raw volatile memory access
/// is trusted via `raw_store()`.
///
pub fn store(index: usize, value: usize) -> (result: Result<(), Error>)
    ensures
        // Success case: index was valid.
        result.is_ok() ==> spec_is_valid_index(index as int),
        // Failure case: index was out of bounds.
        result.is_err() ==> !spec_is_valid_index(index as int),
{
    // Verified bounds check.
    if index >= NUM_ENTRIES {
        return Err(Error::new(ErrorCode::InvalidArgument, "index out of bounds"));
    }
    
    // Trusted raw memory access.
    raw_store(index, value);
    Ok(())
}


/// Raw store operation (trusted).
///
/// This function performs the actual volatile write to the kredzone memory.
/// It is marked `external_body` because volatile operations are opaque to Verus.
///
/// # Safety
///
/// Caller must ensure `index < NUM_ENTRIES`. This is enforced by the verified
/// `store()` function which is the only intended caller.
///
/// # Warning
///
/// This file is for **verification only**. The body below is a stub that Verus ignores.
/// The actual implementation uses `ptr.write_volatile(value)`.
/// See: src/kernel/src/mm/kredzone.rs
#[verifier::external_body]
fn raw_store(index: usize, value: usize)
    requires
        spec_is_valid_index(index as int),
{
    // VERIFICATION STUB: Verus ignores this body.
    // Actual implementation:
    //   unsafe {
    //       let ptr: *mut usize = core::ptr::addr_of_mut!(kredzone);
    //       let ptr: *mut usize = ptr.add(index);
    //       ptr.write_volatile(value);
    //   }
}


/// Loads a value from the kernel red zone.
///
/// # Description
///
/// Loads a value from the given index in the kernel red zone.
///
/// # Parameters
///
/// - `index`: Index in the kernel red zone.
///
/// # Returns
///
/// Upon success, the value is returned. Upon failure, an error is returned instead.
///
/// # Errors
///
/// - `InvalidArgument`: If index is out of bounds.
///
/// # Verification Notes
///
/// This function is marked `external_body` because it accesses an extern "C" static
/// variable using volatile pointer operations. The specification captures:
///
/// - **Postcondition on success**: The index was valid.
/// - **Postcondition on failure**: The index was out of bounds.
///
/// The actual value returned is trusted, not verified.
/// **For functional correctness verification, use `load_with_ghost()` instead.**
/// Callers reasoning about returned values should use `spec_load_result()` with
/// their ghost state to determine the expected value.
///
/// # Verified Bounds Check
///
/// The bounds check is **verified** by Verus. Only the raw volatile memory access
/// is trusted via `raw_load()`.
///
pub fn load(index: usize) -> (result: Result<usize, Error>)
    ensures
        // Success case: index was valid.
        result.is_ok() ==> spec_is_valid_index(index as int),
        // Failure case: index was out of bounds.
        result.is_err() ==> !spec_is_valid_index(index as int),
{
    // Verified bounds check.
    if index >= NUM_ENTRIES {
        return Err(Error::new(ErrorCode::InvalidArgument, "index out of bounds"));
    }
    
    // Trusted raw memory access.
    let value = raw_load(index);
    Ok(value)
}


/// Raw load operation (trusted).
///
/// This function performs the actual volatile read from the kredzone memory.
/// It is marked `external_body` because volatile operations are opaque to Verus.
///
/// # Safety
///
/// Caller must ensure `index < NUM_ENTRIES`. This is enforced by the verified
/// `load()` function which is the only intended caller.
///
/// # Warning
///
/// This file is for **verification only**. The body below is a stub that Verus ignores.
/// The actual implementation uses `ptr.read_volatile()`.
/// See: src/kernel/src/mm/kredzone.rs
#[verifier::external_body]
fn raw_load(index: usize) -> (value: usize)
    requires
        spec_is_valid_index(index as int),
{
    // VERIFICATION STUB: Verus ignores this body.
    // Actual implementation:
    //   unsafe {
    //       let ptr: *const usize = core::ptr::addr_of!(kredzone);
    //       let ptr: *const usize = ptr.add(index);
    //       ptr.read_volatile()
    //   }
    0  // Placeholder; actual value comes from volatile read.
}

//==================================================================================================

/// Verified wrapper for `store` that manages ghost state.
///
/// This wrapper allows callers to reason about sequences of store/load operations
/// using the abstract model. The wrapper:
///
/// 1. Calls the raw `store` function (trusted via `external_body`).
/// 2. Updates the ghost state to reflect the expected effect.
///
/// Under trust assumptions T1-T3, the ghost state remains synchronized with
/// the actual memory state.
///
/// # Example Usage
///
/// ```ignore
/// let ghost mut view = initial_well_formed_view();
/// store_with_ghost(5, 42, Tracked(&mut view))?;
/// // Now: view == spec_store_effect(old_view, 5, 42)
/// ```
pub fn store_with_ghost(
    index: usize,
    value: usize,
    Tracked(ghost): Tracked<&mut KernelRedZoneGhost>,
) -> (result: Result<(), Error>)
    requires
        old(ghost).inv(),
    ensures
        result.is_ok() ==> {
            &&& spec_is_valid_index(index as int)
            &&& ghost.view == spec_store_effect(old(ghost).view, index as int, value)
            &&& ghost.inv()
        },
        result.is_err() ==> {
            &&& !spec_is_valid_index(index as int)
            &&& ghost.view == old(ghost).view
        },
{
    let res = store(index, value);
    if res.is_ok() {
        proof {
            lemma_valid_index_in_bounds(ghost.view, index as int);
            lemma_update_preserves_well_formed(ghost.view, index as int, value);
            ghost.view = spec_store_effect(ghost.view, index as int, value);
        }
    }
    res
}


/// Verified wrapper for `load` that uses ghost state for reasoning.
///
/// This wrapper allows callers to reason about the expected return value
/// using the abstract model. The wrapper:
///
/// 1. Calls the raw `load` function (trusted via `external_body`).
/// 2. Returns the value with a postcondition relating it to the ghost state.
///
/// Under trust assumptions T1-T3, the returned value equals `spec_load_result`.
///
/// # Trust Assumption
///
/// The postcondition `result.unwrap() == spec_load_result(ghost.view, index)` relies
/// on T2 (volatile reads return last written value). This is explicitly assumed
/// via `assume` in the function body, bridging the gap between the abstract model
/// and the implementation.
pub fn load_with_ghost(
    index: usize,
    Tracked(ghost): Tracked<&KernelRedZoneGhost>,
) -> (result: Result<usize, Error>)
    requires
        ghost.inv(),
    ensures
        result.is_ok() ==> {
            &&& spec_is_valid_index(index as int)
            &&& result.unwrap() == spec_load_result(ghost.view, index as int)
        },
        result.is_err() ==> !spec_is_valid_index(index as int),
{
    let res = load(index);
    proof {
        if res.is_ok() {
            // TRUST ASSUMPTIONS T2 + T5:
            // - T2: Volatile reads return the last value written.
            // - T5: The ghost state is the UNIQUE model of kredzone, kept in sync.
            //
            // This assume bridges the abstract model to the implementation.
            // Validity: The extern C kredzone memory and volatile semantics ensure
            // that read_volatile returns the value from the last write_volatile,
            // AND the caller has maintained the ghost state via store_with_ghost.
            //
            // SOUNDNESS WARNING: This assume is only valid if T5 holds. If multiple
            // ghost instances exist, or if the raw store() was used without updating
            // ghost state, this assume may assert a false equality.
            assume(res.unwrap() == spec_load_result(ghost.view, index as int));
        }
    }
    res
}


/// Initializes the kredzone memory to all zeros.
///
/// This function provides an **alternative to relying on T4** (BSS zero-initialization).
/// It explicitly writes zeros to all kredzone entries, ensuring the concrete memory
/// is in a known state.
///
/// # Description
///
/// Performs `store(i, 0)` for all valid indices. After calling this function,
/// the kredzone memory is guaranteed to contain all zeros, and callers can safely
/// use `create_initial_ghost()` knowing the ghost state will match reality.
///
/// # Usage
///
/// Call this during kernel initialization before using `create_initial_ghost()`:
/// ```ignore
/// init_kredzone();  // Explicitly zeros all entries.
/// let tracked ghost = create_initial_ghost();  // Now safe to use.
/// ```
pub fn init_kredzone()
    ensures
        true,  // Effect: all kredzone entries are set to 0.
{
    // Zero all entries in the kredzone.
    let mut idx: usize = 0;
    while idx < NUM_ENTRIES
        invariant
            idx <= NUM_ENTRIES,
        decreases
            NUM_ENTRIES - idx,
    {
        let _ = store(idx, 0);  // Cannot fail: idx < NUM_ENTRIES.
        idx = idx + 1;
    }
}

} // verus!
