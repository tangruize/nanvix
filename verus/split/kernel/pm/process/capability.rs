// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Capabilities Implementation
//!
//! A bitfield type that tracks which capabilities are granted to a process.
//!
//! ## Verified Properties
//!
//! - Default capabilities have no bits set (empty bitfield).
//! - `set` guarantees the target bit is set afterward.
//! - `clear` guarantees the target bit is cleared afterward.
//! - `has` correctly tests the target bit.
//! - `set` and `clear` preserve all other capability bits.
//! - `set` on an already-set bit is idempotent.
//! - `clear` on an already-clear bit is idempotent.
//! - Well-formedness (only valid bits 0..=4 set) is preserved by all operations.
//! - `set` then `clear` is a roundtrip (restores original when bit was clear).
//! - `clear` then `set` is a roundtrip (restores original when bit was set).
//! - Explicit mask values match the original `1 << discriminant` formula.
//! - The Capability enum is closed: every instance is one of the 5 known variants.
//! - View equality implies bitfield equality.
//!
//! ## Verification Additions
//!
//! The following items are added for verification and do not exist in the
//! original source (`src/kernel/src/pm/process/capability.rs`):
//! - `CapabilitiesView` and `View` impl: abstract view for composability.
//! - `spec_default`: spec-level constructor for the default value.
//! - `PartialEq`/`Eq` derives: required by Verus for equality reasoning.
//! - Explicit `Default` impl: replaces `#[derive(Default)]` for Verus compatibility.
//! - `to_mask`: exec-level mask computation; uses explicit match rather than
//!   `1 << discriminant` to avoid dependence on enum layout or `#[repr]`
//!   annotations. Equivalence to the original formula is proven by
//!   `lemma_mask_matches_discriminant`.
//! - `pub bits` field: required by Verus for spec-level field access in
//!   `pub open spec fn` definitions. Verus requires that field access in a
//!   `pub open spec fn` be well-formed everywhere the spec fn is visible;
//!   since the spec fns are `pub`, the field must also be `pub`. This is the
//!   established pattern in the Nanvix verification crate (cf. `ProcessIdentifier.value`).
//!   The original uses a private tuple field `(u8)`. External code should use
//!   the `set`/`clear`/`has` API rather than accessing `bits` directly. The
//!   `wf()` predicate serves as a module-level invariant that all API-constructed
//!   values satisfy and all operations preserve.
//!
//! ## Closed-World Assumption
//!
//! The `Capability` enum is exhaustively verified in `sys::pm::capability` with
//! exactly 5 variants (discriminants 0..=4). The explicit `match` in `spec_mask`
//! and `to_mask` covers all variants; adding a new variant to `Capability` would
//! cause a compile-time error in every `match` expression throughout both the
//! original and verified code. The `lemma_enum_is_closed` proof additionally
//! verifies that every `Capability` instance is one of the 5 known variants.
//!
//! ## Trust Boundary
//!
//! The `Capability` enum is defined in the verified `sys::pm::capability` module.
//! Its discriminant properties are used here via `spec_discriminant()` and the
//! associated proof lemmas. No `external_body` or `assume` is used.

use crate::kernel::pm::sys::capability::Capability;
use vstd::prelude::*;

// Include specifications.
include!("capability.spec.rs");

// Include proofs.
include!("capability.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A bitfield type that tracks which capabilities are granted to a process.
///
/// # Description
///
/// Each bit in the underlying `u8` corresponds to a `Capability` variant.
/// Bit `i` is set if and only if the capability with discriminant `i` is granted.
///
/// # Note
///
/// The `bits` field is `pub` because Verus requires public fields for spec-level
/// access in `pub open spec fn` definitions. This is a Verus constraint: field
/// expressions in `pub open spec fn` must be well-formed everywhere the spec fn
/// is visible. The original source uses a private tuple struct `Capabilities(u8)`.
/// External code should use the `set`/`clear`/`has` API rather than accessing
/// `bits` directly. The `wf()` predicate is the module-level invariant:
/// all API-constructed values satisfy it and all operations preserve it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// The raw bitfield value.
    bits: u8,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Capabilities {
    /// Spec function: returns the default (empty) Capabilities value.
    pub open spec fn spec_default() -> Capabilities {
        Capabilities { bits: 0u8 }
    }

    /// Computes the bitmask for a given capability.
    ///
    /// # Description
    ///
    /// Uses explicit match rather than `1 << discriminant` to avoid dependence
    /// on enum layout or `#[repr]` annotations. The equivalence to the original
    /// source's shift-based formula is proven by `lemma_mask_matches_discriminant`.
    /// Adding a new variant to `Capability` would cause a compile-time error here.
    ///
    /// # Parameters
    ///
    /// - `capability`: The capability whose mask to compute.
    ///
    /// # Returns
    ///
    /// A single-bit u8 mask for the capability.
    fn to_mask(capability: Capability) -> (result: u8)
        ensures
            result == Self::spec_mask(capability),
    {
        match capability {
            Capability::ExceptionControl => 1u8,
            Capability::InterruptControl => 2u8,
            Capability::IoManagement => 4u8,
            Capability::MemoryManagement => 8u8,
            Capability::ProcessManagement => 16u8,
        }
    }

    /// Creates a new, empty Capabilities value.
    ///
    /// # Returns
    ///
    /// A Capabilities with no bits set.
    pub fn new() -> (result: Capabilities)
        ensures
            result.spec_bits() == 0u8,
            result == Capabilities::spec_default(),
            result.wf(),
    {
        proof {
            assert(0u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
        }
        Capabilities { bits: 0u8 }
    }

    /// Sets a capability bit.
    ///
    /// # Parameters
    ///
    /// - `capability`: The capability to grant.
    ///
    /// # Ensures
    ///
    /// - The target bit is set in the result.
    /// - All other bits are unchanged (bitfield equals `old | mask`).
    /// - If the input was well-formed, the output is well-formed.
    pub fn set(&mut self, capability: Capability)
        ensures
            self.spec_bits() == old(self).spec_set(capability),
            self.spec_has(capability),
            old(self).wf() ==> self.wf(),
    {
        let mask: u8 = Self::to_mask(capability);
        let ghost pre = *self;
        self.bits = self.bits | mask;

        proof {
            Capabilities::lemma_set_then_has(pre, capability);
            if pre.wf() {
                Capabilities::lemma_set_preserves_wf(pre, capability);
            }
        }
    }

    /// Clears a capability bit.
    ///
    /// # Parameters
    ///
    /// - `capability`: The capability to revoke.
    ///
    /// # Ensures
    ///
    /// - The target bit is cleared in the result.
    /// - All other bits are unchanged (bitfield equals `old & !mask`).
    /// - If the input was well-formed, the output is well-formed.
    pub fn clear(&mut self, capability: Capability)
        ensures
            self.spec_bits() == old(self).spec_clear(capability),
            !self.spec_has(capability),
            old(self).wf() ==> self.wf(),
    {
        let mask: u8 = Self::to_mask(capability);
        let ghost pre = *self;
        self.bits = self.bits & !mask;

        proof {
            Capabilities::lemma_clear_then_not_has(pre, capability);
            if pre.wf() {
                Capabilities::lemma_clear_preserves_wf(pre, capability);
            }
        }
    }

    /// Tests whether a capability bit is set.
    ///
    /// # Parameters
    ///
    /// - `capability`: The capability to test.
    ///
    /// # Returns
    ///
    /// `true` if the capability is granted, `false` otherwise.
    pub fn has(&self, capability: Capability) -> (result: bool)
        ensures
            result == self.spec_has(capability),
    {
        let mask: u8 = Self::to_mask(capability);
        (self.bits & mask) != 0u8
    }
}

//==================================================================================================
// Trait Implementations
//==================================================================================================

impl Default for Capabilities {
    /// Returns the default Capabilities (no capabilities granted).
    fn default() -> (result: Capabilities)
        ensures
            result.spec_bits() == 0u8,
            result == Capabilities::spec_default(),
            result.wf(),
    {
        proof {
            assert(0u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
        }
        Capabilities { bits: 0u8 }
    }
}

} // verus!
