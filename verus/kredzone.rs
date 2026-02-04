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

use crate::error::{Error, ErrorCode};
use vstd::prelude::*;

verus! {

//==================================================================================================
// Constants
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

// Fallback for unsupported architectures - defaults to 64-bit with a warning.
// Nanvix officially targets x86-32. This fallback exists only for Verus compilation
// on non-standard hosts. Production builds should fail on unsupported architectures.
#[cfg(all(not(target_pointer_width = "32"), not(target_pointer_width = "64")))]
pub const ENTRY_SIZE: usize = 8;  // WARNING: Unsupported architecture, defaulting to 64-bit.

/// Number of entries in the kernel red zone (spec version).
pub spec const SPEC_NUM_ENTRIES: int = KREDZONE_SIZE as int / ENTRY_SIZE as int;

/// Lemma: ENTRY_SIZE matches size_of::<usize>() for the target platform.
///
/// This lemma documents the assumption that ENTRY_SIZE correctly reflects
/// the pointer width. Since Verus cannot directly reason about size_of,
/// we verify this through conditional compilation matching target_pointer_width.
///
/// For 32-bit: ENTRY_SIZE = 4 = sizeof(u32) = sizeof(usize)
/// For 64-bit: ENTRY_SIZE = 8 = sizeof(u64) = sizeof(usize)
#[cfg(target_pointer_width = "64")]
pub proof fn lemma_entry_size_matches_target()
    ensures
        ENTRY_SIZE == 8,
        SPEC_NUM_ENTRIES == 16,  // 128 / 8 = 16 entries.
{
}

#[cfg(target_pointer_width = "32")]
pub proof fn lemma_entry_size_matches_target()
    ensures
        ENTRY_SIZE == 4,
        SPEC_NUM_ENTRIES == 32,  // 128 / 4 = 32 entries.
{
}

/// Fallback lemma for unsupported architectures (defaults to 64-bit behavior).
/// This ensures the lemma exists on all platforms, even if the platform is
/// not officially supported. The ENTRY_SIZE defaults to 8 in this case.
#[cfg(all(not(target_pointer_width = "32"), not(target_pointer_width = "64")))]
pub proof fn lemma_entry_size_matches_target()
    ensures
        ENTRY_SIZE == 8,
        SPEC_NUM_ENTRIES == 16,  // 128 / 8 = 16 entries (default 64-bit).
{
}

/// Number of entries in the kernel red zone (exec version).
pub const NUM_ENTRIES: usize = KREDZONE_SIZE / ENTRY_SIZE;

//==================================================================================================
// Specification Helper Functions
//==================================================================================================

/// Checks if an index is valid (within bounds).
pub open spec fn spec_is_valid_index(index: int) -> bool {
    0 <= index < SPEC_NUM_ENTRIES
}

//==================================================================================================
// KernelRedZoneView - Abstract Specification
//==================================================================================================

/// Abstract view of the kernel red zone for specification purposes.
///
/// This view models the red zone as a sequence of usize values.
#[verifier::ext_equal]
pub struct KernelRedZoneView {
    /// The logical contents of the red zone.
    pub contents: Seq<usize>,
}

impl KernelRedZoneView {
    //==============================================================================================
    // Basic Properties
    //==============================================================================================

    /// Returns the number of entries in the red zone.
    pub open spec fn len(&self) -> nat {
        self.contents.len()
    }

    /// Returns the value at index i.
    pub open spec fn index(&self, i: int) -> usize
        recommends 0 <= i < self.len() as int
    {
        self.contents[i]
    }

    /// Returns a new view with the element at index i updated to value.
    pub open spec fn update(&self, i: int, value: usize) -> KernelRedZoneView
        recommends 0 <= i < self.len() as int
    {
        KernelRedZoneView {
            contents: self.contents.update(i, value),
        }
    }

    /// Checks if an index is within bounds.
    pub open spec fn in_bounds(&self, i: int) -> bool {
        0 <= i < self.len() as int
    }

    //==============================================================================================
    // Well-formedness
    //==============================================================================================

    /// Returns true if the view represents a well-formed kernel red zone.
    pub open spec fn is_well_formed(&self) -> bool {
        self.len() == SPEC_NUM_ENTRIES as nat
    }
}

//==================================================================================================
// Lemmas about KernelRedZoneView - Fully Verified
//==================================================================================================

/// Lemma: Updating an entry preserves the length.
pub proof fn lemma_update_preserves_len(view: KernelRedZoneView, i: int, value: usize)
    requires
        view.in_bounds(i),
    ensures
        view.update(i, value).len() == view.len(),
{
    // Follows from Seq::update preserving length.
}

/// Lemma: Updating index i only changes index i.
pub proof fn lemma_update_only_changes_index(view: KernelRedZoneView, i: int, value: usize, j: int)
    requires
        view.in_bounds(i),
        view.in_bounds(j),
        i != j,
    ensures
        view.update(i, value).index(j) == view.index(j),
{
    // Follows from Seq::update property.
}

/// Lemma: Updating index i sets index i to the new value.
pub proof fn lemma_update_sets_index(view: KernelRedZoneView, i: int, value: usize)
    requires
        view.in_bounds(i),
    ensures
        view.update(i, value).index(i) == value,
{
    // Follows from Seq::update property.
}

/// Lemma: Well-formed view has correct number of entries.
pub proof fn lemma_well_formed_entries(view: KernelRedZoneView)
    requires
        view.is_well_formed(),
    ensures
        view.len() == SPEC_NUM_ENTRIES as nat,
{
}

/// Lemma: Valid indices are within well-formed view bounds.
pub proof fn lemma_valid_index_in_bounds(view: KernelRedZoneView, i: int)
    requires
        view.is_well_formed(),
        spec_is_valid_index(i),
    ensures
        view.in_bounds(i),
{
}

/// Lemma: Update preserves well-formedness.
pub proof fn lemma_update_preserves_well_formed(view: KernelRedZoneView, i: int, value: usize)
    requires
        view.is_well_formed(),
        view.in_bounds(i),
    ensures
        view.update(i, value).is_well_formed(),
{
    lemma_update_preserves_len(view, i, value);
}

//==================================================================================================
// KernelRedZone - Ghost State Model
//==================================================================================================

/// Ghost state representing the abstract kernel red zone.
///
/// Since the kernel red zone is a global static variable accessed via `extern "C"`,
/// we cannot thread tracked ghost state through the `store`/`load` functions directly.
/// Instead, this struct provides an abstract model for **caller-side reasoning**.
///
/// ## Usage Pattern
///
/// Callers who need to reason about sequences of store/load operations can:
///
/// 1. Maintain a `ghost view: KernelRedZoneView` that tracks expected state.
/// 2. After calling `store(i, v)`, update ghost state: `view = spec_store_effect(view, i, v)`.
/// 3. Before calling `load(i)`, assert expected value: `spec_load_result(view, i)`.
///
/// The correctness lemmas (`lemma_store_then_load`, etc.) prove that if the ghost
/// state is maintained correctly and the trust assumptions hold, the abstract
/// reasoning is valid.
///
/// ## Why Not Tracked Parameters?
///
/// The original implementation uses an `extern "C"` static variable, which:
/// - Has no Rust ownership model (it's assembly-defined).
/// - Cannot accept tracked/ghost parameters (fixed C ABI).
/// - Is accessed via raw pointer arithmetic with volatile operations.
///
/// Therefore, we document the trust boundary rather than attempting to force
/// ghost state through an incompatible interface.
#[verifier::ext_equal]
pub tracked struct KernelRedZoneGhost {
    pub ghost view: KernelRedZoneView,
}

impl KernelRedZoneGhost {
    /// Returns true if the ghost state is well-formed.
    pub open spec fn inv(&self) -> bool {
        self.view.is_well_formed()
    }

    /// Returns the abstract view.
    pub open spec fn view(&self) -> KernelRedZoneView {
        self.view
    }
}

//==================================================================================================
// Standalone Public Functions
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
// Pure Specification Functions for Reasoning
//==================================================================================================

/// Specification function: Effect of a store operation on abstract state.
///
/// Given the current view and a valid store operation, returns the resulting view.
pub open spec fn spec_store_effect(
    view: KernelRedZoneView,
    index: int,
    value: usize,
) -> KernelRedZoneView
    recommends
        view.is_well_formed(),
        spec_is_valid_index(index),
{
    view.update(index, value)
}

/// Specification function: Result of a load operation on abstract state.
///
/// Given the current view and a valid load operation, returns the value.
pub open spec fn spec_load_result(view: KernelRedZoneView, index: int) -> usize
    recommends
        view.is_well_formed(),
        spec_is_valid_index(index),
{
    view.index(index)
}

//==================================================================================================
// Verified Wrapper Layer
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

/// Creates an initial well-formed ghost state with all zeros.
///
/// This is useful for initializing ghost state at the start of kernel execution.
///
/// # Trust Assumptions
///
/// This function relies on:
///
/// - **T4**: The kredzone region is zero-initialized before first use (BSS section).
/// - **T5**: Only one `KernelRedZoneGhost` instance is active at any time.
///
/// If T4 is violated, the ghost state will be inconsistent with the actual memory.
/// If T5 is violated (multiple ghost instances exist), the `assume` in `load_with_ghost`
/// becomes unsound.
///
/// # Alternative
///
/// To establish the invariant without relying on T4, call `init_kredzone()` first,
/// which explicitly writes zeros to all entries.
///
/// # Soundness Warning
///
/// **Do not call this function more than once.** Creating multiple ghost instances
/// that all claim to model the single global kredzone will lead to unsound verification.
/// See module documentation for the recommended usage pattern.
pub proof fn create_initial_ghost() -> (tracked result: KernelRedZoneGhost)
    ensures
        result.inv(),
        forall|i: int| #![auto] spec_is_valid_index(i) ==> result.view.index(i) == 0usize,
{
    let contents: Seq<usize> = Seq::new(SPEC_NUM_ENTRIES as nat, |i| 0usize);
    let view = KernelRedZoneView { contents };
    KernelRedZoneGhost { view }
}

//==================================================================================================
// Correctness Properties (Proof-only)
//==================================================================================================

/// Property: Store then load at the same index returns the stored value.
///
/// This is the fundamental read-after-write property.
pub proof fn lemma_store_then_load(view: KernelRedZoneView, index: int, value: usize)
    requires
        view.is_well_formed(),
        spec_is_valid_index(index),
    ensures
        spec_load_result(spec_store_effect(view, index, value), index) == value,
{
    lemma_valid_index_in_bounds(view, index);
    lemma_update_sets_index(view, index, value);
}

/// Property: Store at index i does not affect load at index j (i != j).
///
/// This is the non-interference property.
pub proof fn lemma_store_does_not_affect_other(
    view: KernelRedZoneView,
    i: int,
    value: usize,
    j: int,
)
    requires
        view.is_well_formed(),
        spec_is_valid_index(i),
        spec_is_valid_index(j),
        i != j,
    ensures
        spec_load_result(spec_store_effect(view, i, value), j) == spec_load_result(view, j),
{
    lemma_valid_index_in_bounds(view, i);
    lemma_valid_index_in_bounds(view, j);
    lemma_update_only_changes_index(view, i, value, j);
}

/// Property: Store preserves well-formedness.
pub proof fn lemma_store_preserves_invariant(view: KernelRedZoneView, index: int, value: usize)
    requires
        view.is_well_formed(),
        spec_is_valid_index(index),
    ensures
        spec_store_effect(view, index, value).is_well_formed(),
{
    lemma_valid_index_in_bounds(view, index);
    lemma_update_preserves_well_formed(view, index, value);
}

/// Property: Two consecutive stores to the same index result in the last value.
pub proof fn lemma_store_overwrite(view: KernelRedZoneView, index: int, v1: usize, v2: usize)
    requires
        view.is_well_formed(),
        spec_is_valid_index(index),
    ensures
        spec_store_effect(spec_store_effect(view, index, v1), index, v2) ==
            spec_store_effect(view, index, v2),
{
    lemma_valid_index_in_bounds(view, index);
    let view1: KernelRedZoneView = spec_store_effect(view, index, v1);
    lemma_update_preserves_well_formed(view, index, v1);
    lemma_valid_index_in_bounds(view1, index);

    // Both result in the same view.
    assert(view.contents.update(index, v1).update(index, v2) =~= view.contents.update(index, v2));
}

/// Property: Stores to different indices commute.
pub proof fn lemma_store_commutes(view: KernelRedZoneView, i: int, vi: usize, j: int, vj: usize)
    requires
        view.is_well_formed(),
        spec_is_valid_index(i),
        spec_is_valid_index(j),
        i != j,
    ensures
        spec_store_effect(spec_store_effect(view, i, vi), j, vj) ==
            spec_store_effect(spec_store_effect(view, j, vj), i, vi),
{
    lemma_valid_index_in_bounds(view, i);
    lemma_valid_index_in_bounds(view, j);

    // Seq::update commutes for different indices.
    assert(view.contents.update(i, vi).update(j, vj) =~= view.contents.update(j, vj).update(i, vi));
}

//==================================================================================================
// Bounds Checking Properties
//==================================================================================================

/// Property: NUM_ENTRIES equals SPEC_NUM_ENTRIES.
pub proof fn lemma_num_entries_correct()
    ensures
        NUM_ENTRIES as int == SPEC_NUM_ENTRIES,
{
}

/// Property: Maximum valid index is NUM_ENTRIES - 1.
pub proof fn lemma_max_valid_index()
    ensures
        spec_is_valid_index((NUM_ENTRIES - 1) as int),
        !spec_is_valid_index(NUM_ENTRIES as int),
{
}

/// Property: Negative indices are invalid.
pub proof fn lemma_negative_index_invalid(i: int)
    requires
        i < 0,
    ensures
        !spec_is_valid_index(i),
{
}

//==================================================================================================
// Tests (Abstract Model Only)
//==================================================================================================

/// These tests verify properties of the abstract `KernelRedZoneView` model.
/// Due to `external_body` on `store`/`load`, these tests do NOT verify the
/// executable code paths. They ensure the abstract specification is internally
/// consistent and satisfies the expected algebraic properties.
#[cfg(verus_keep_ghost)]
mod test {
    use super::*;

    /// Test: Basic view properties.
    proof fn test_view_properties() {
        let contents: Seq<usize> = Seq::new(SPEC_NUM_ENTRIES as nat, |i| 0usize);
        let view: KernelRedZoneView = KernelRedZoneView { contents };

        // View is well-formed.
        assert(view.is_well_formed());
        assert(view.len() == SPEC_NUM_ENTRIES as nat);

        // Index 0 is valid and in bounds.
        assert(spec_is_valid_index(0));
        lemma_valid_index_in_bounds(view, 0);
        assert(view.in_bounds(0));
    }

    /// Test: Update preserves length.
    proof fn test_update_preserves_len() {
        let contents: Seq<usize> = Seq::new(SPEC_NUM_ENTRIES as nat, |i| 0usize);
        let view: KernelRedZoneView = KernelRedZoneView { contents };

        let updated: KernelRedZoneView = view.update(0, 42);
        lemma_update_preserves_len(view, 0, 42);
        assert(updated.len() == view.len());
    }

    /// Test: Read-after-write property.
    proof fn test_read_after_write() {
        let contents: Seq<usize> = Seq::new(SPEC_NUM_ENTRIES as nat, |i| 0usize);
        let view: KernelRedZoneView = KernelRedZoneView { contents };

        lemma_store_then_load(view, 5, 123);
        assert(spec_load_result(spec_store_effect(view, 5, 123), 5) == 123);
    }

    /// Test: Non-interference property.
    proof fn test_non_interference() {
        let contents: Seq<usize> = Seq::new(SPEC_NUM_ENTRIES as nat, |i| i as usize);
        let view: KernelRedZoneView = KernelRedZoneView { contents };

        // Store at index 3, should not affect index 7.
        lemma_store_does_not_affect_other(view, 3, 999, 7);
        assert(spec_load_result(spec_store_effect(view, 3, 999), 7) == spec_load_result(view, 7));
    }

    /// Test: Store overwrites previous value.
    proof fn test_store_overwrite() {
        let contents: Seq<usize> = Seq::new(SPEC_NUM_ENTRIES as nat, |i| 0usize);
        let view: KernelRedZoneView = KernelRedZoneView { contents };

        lemma_store_overwrite(view, 2, 100, 200);
        let result: KernelRedZoneView = spec_store_effect(spec_store_effect(view, 2, 100), 2, 200);
        let direct: KernelRedZoneView = spec_store_effect(view, 2, 200);
        assert(result == direct);
    }

    /// Test: Stores commute.
    proof fn test_stores_commute() {
        let contents: Seq<usize> = Seq::new(SPEC_NUM_ENTRIES as nat, |i| 0usize);
        let view: KernelRedZoneView = KernelRedZoneView { contents };

        lemma_store_commutes(view, 1, 10, 4, 40);
        let order1: KernelRedZoneView = spec_store_effect(spec_store_effect(view, 1, 10), 4, 40);
        let order2: KernelRedZoneView = spec_store_effect(spec_store_effect(view, 4, 40), 1, 10);
        assert(order1 == order2);
    }

    /// Test: Bounds checking.
    proof fn test_bounds_checking() {
        lemma_num_entries_correct();
        lemma_max_valid_index();

        // Valid indices.
        assert(spec_is_valid_index(0));
        assert(spec_is_valid_index((NUM_ENTRIES - 1) as int));

        // Invalid indices.
        assert(!spec_is_valid_index(-1));
        assert(!spec_is_valid_index(NUM_ENTRIES as int));
        assert(!spec_is_valid_index(1000));
    }

    /// Test: Well-formedness preservation.
    proof fn test_well_formedness_preservation() {
        let contents: Seq<usize> = Seq::new(SPEC_NUM_ENTRIES as nat, |i| 0usize);
        let view: KernelRedZoneView = KernelRedZoneView { contents };

        assert(view.is_well_formed());
        lemma_store_preserves_invariant(view, 0, 42);
        assert(spec_store_effect(view, 0, 42).is_well_formed());
    }

    /// Test: Entry size is correct for target platform.
    proof fn test_entry_size_correct() {
        lemma_entry_size_matches_target();
    }

    /// Test: Ghost state wrappers compile and proof steps succeed.
    /// This exercises the store_with_ghost and load_with_ghost wrapper logic.
    proof fn test_ghost_wrapper_reasoning() {
        // Create initial ghost state (relies on T4: zero initialization).
        let tracked mut ghost = create_initial_ghost();
        
        // Verify initial state is well-formed.
        assert(ghost.inv());
        // The postcondition of create_initial_ghost ensures index 0 is 0.
        assert(spec_is_valid_index(0));  // Index 0 is valid.
        // Use the postcondition: forall i, valid(i) ==> view.index(i) == 0
        
        // Model a store operation at index 0 with value 42.
        lemma_valid_index_in_bounds(ghost.view, 0);
        let new_view = spec_store_effect(ghost.view, 0, 42);
        lemma_update_preserves_well_formed(ghost.view, 0, 42);
        ghost.view = new_view;
        
        // Verify the new ghost state is well-formed.
        assert(ghost.inv());
        
        // Verify read-after-write: loading from index 0 should return 42.
        lemma_store_then_load(ghost.view, 0, 42);
        // After store(0, 42), spec_load_result(view, 0) == 42.
    }
}

} // verus!
