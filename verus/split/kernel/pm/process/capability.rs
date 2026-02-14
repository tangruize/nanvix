// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Capabilities Implementation
//!
//! A bitfield type that tracks which capabilities are granted to a process.
//!
//! ## Abstraction Levels
//!
//! This module provides two levels of specification:
//! - **Bit-level**: `spec_bits`, `spec_has`, `spec_set`, `spec_clear` — directly
//!   model the u8 bitfield implementation.
//! - **Set-level**: `spec_as_set`, `spec_granted`, `spec_set_contains`,
//!   `spec_set_insert`, `spec_set_remove` — model capabilities as a `Set<Capability>`.
//!
//! Downstream modules should prefer the set-level abstraction. The bit-level
//! specs exist for internal proof obligations and are bridged to the set-level
//! via `lemma_has_iff_set_contains`, `lemma_set_insert_matches_bit_set`, and
//! `lemma_clear_remove_matches_bit_clear`.
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
//!   `pub open spec fn` definitions. Verus enforces that field expressions in
//!   `pub open spec fn` must be well-formed everywhere the spec fn is visible.
//!   Because these spec fns are `pub` (needed for downstream compositional
//!   verification), the field must also be `pub`. Alternatives were tested and
//!   rejected:
//!   - `pub(crate)`: Verus error — "must be well-formed everywhere, which is
//!     wider than `lib`".
//!   - `pub(super)`: Same Verus error.
//!   - Private field + `closed spec fn`: Verus error — constructors in `ensures`
//!     clauses of pub functions must also be well-formed everywhere.
//!   This is the established pattern in the Nanvix verification crate
//!   (cf. `ProcessIdentifier.value` in `kernel/pm/sys/pid.rs`).
//!   The original uses a private tuple field `(u8)`. External code should use
//!   the `set`/`clear`/`has` API rather than accessing `bits` directly.
//!
//! ## Invariant Enforcement
//!
//! The `wf()` predicate (only bits 0..=4 may be set) serves as the module-level
//! invariant. While the `pub bits` field permits constructing non-`wf` values,
//! the invariant is enforced by proof obligations:
//! - All constructors (`new`, `default`) guarantee `wf()` in their postconditions.
//! - `set`/`clear` require `old(self).wf()` and guarantee `self.wf()` unconditionally.
//! - `has` requires `self.wf()`.
//! - `lemma_api_preserves_wf` proves the inductive step for any operation sequence.
//! - Downstream modules that require `wf()` can assert it as a precondition,
//!   knowing that any value produced through the API satisfies it.
//! This pattern matches the Nanvix verification crate convention where `wf()`
//! is a proof-level obligation, not a runtime-enforced type invariant.
//!
//! **Known deviation:** The verified executable interface is strictly more
//! permissive than the original source because the `pub bits` field allows
//! direct construction and mutation that bypasses the `set`/`clear` API. This
//! is an unavoidable consequence of Verus's visibility rules (see above).
//! The verification guarantees correctness for the API-reachable state space;
//! values constructed by directly writing to `bits` are outside the verified
//! contract and are not guaranteed to satisfy `wf()`.
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
/// The `bits` field is `pub` due to a Verus tooling constraint: `pub open spec fn`
/// definitions require field expressions to be well-formed at all visibility scopes,
/// and `pub(crate)`, `pub(super)`, and private fields all trigger Verus errors.
/// The original source uses a private tuple struct `Capabilities(u8)`. External
/// code must use `set`/`clear`/`has` rather than accessing `bits` directly.
/// The `wf()` predicate is the proof-level invariant: all API-constructed values
/// satisfy it and all operations conditionally preserve it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// The raw bitfield value.
    pub bits: u8,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Capabilities {
    /// Spec function: returns the default (empty) Capabilities value.
    ///
    /// Closed per methodology Step 3: avoids exposing the internal
    /// representation (`bits: 0u8`). Properties are exposed through
    /// lemmas (`lemma_default_is_empty`, `lemma_default_empty_set`).
    pub closed spec fn spec_default() -> Capabilities {
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
            result == Self::spec_pow2_mask(capability.spec_discriminant()),
    {
        proof {
            Capabilities::lemma_mask_matches_discriminant(capability);
        }
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
    /// A Capabilities with no capabilities granted.
    pub fn new() -> (result: Capabilities)
        ensures
            result.wf(),
            result@.granted =~= Set::<Capability>::empty(),
    {
        proof {
            reveal(Capabilities::wf);
            reveal(Capabilities::spec_default);
            assert(0u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
            Capabilities::lemma_default_empty_set();
        }
        Capabilities { bits: 0u8 }
    }

    /// Grants a capability.
    ///
    /// # Parameters
    ///
    /// - `capability`: The capability to grant.
    pub fn set(&mut self, capability: Capability)
        requires
            old(self).wf(),
        ensures
            self.wf(),
            self@.granted =~= old(self)@.granted.insert(capability),
    {
        let mask: u8 = Self::to_mask(capability);
        let ghost pre = *self;
        self.bits = self.bits | mask;

        proof {
            Capabilities::lemma_set_preserves_wf(pre, capability);
            Capabilities::lemma_set_insert_matches_bit_set(pre, capability);
            assert forall |c: Capability| self.spec_as_set().contains(c) <==>
                (pre.spec_as_set().contains(c) || c == capability) by {
                self.lemma_has_iff_set_contains(c);
                pre.lemma_has_iff_set_contains(c);
            }
        }
    }

    /// Revokes a capability.
    ///
    /// # Parameters
    ///
    /// - `capability`: The capability to revoke.
    pub fn clear(&mut self, capability: Capability)
        requires
            old(self).wf(),
        ensures
            self.wf(),
            self@.granted =~= old(self)@.granted.remove(capability),
    {
        let mask: u8 = Self::to_mask(capability);
        let ghost pre = *self;
        self.bits = self.bits & !mask;

        proof {
            Capabilities::lemma_clear_preserves_wf(pre, capability);
            Capabilities::lemma_clear_remove_matches_bit_clear(pre, capability);
            assert forall |c: Capability| self.spec_as_set().contains(c) <==>
                (pre.spec_as_set().contains(c) && c != capability) by {
                self.lemma_has_iff_set_contains(c);
                pre.lemma_has_iff_set_contains(c);
            }
        }
    }

    /// Tests whether a capability is granted.
    ///
    /// # Parameters
    ///
    /// - `capability`: The capability to test.
    ///
    /// # Returns
    ///
    /// `true` if the capability is granted, `false` otherwise.
    pub fn has(&self, capability: Capability) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self@.granted.contains(capability),
    {
        proof {
            self.lemma_has_iff_set_contains(capability);
        }
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
            result.wf(),
            result@.granted =~= Set::<Capability>::empty(),
    {
        proof {
            reveal(Capabilities::wf);
            reveal(Capabilities::spec_default);
            assert(0u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
            Capabilities::lemma_default_empty_set();
        }
        Capabilities { bits: 0u8 }
    }
}

} // verus!
