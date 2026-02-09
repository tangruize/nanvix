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
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Capabilities(u8);

//==================================================================================================
// Implementations
//==================================================================================================

impl Capabilities {
    /// Spec function: returns the default (empty) Capabilities value.
    pub open spec fn spec_default() -> Capabilities {
        Capabilities(0u8)
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
    {
        Capabilities(0u8)
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
    /// - All other bits are unchanged.
    pub fn set(&mut self, capability: Capability)
        ensures
            self.spec_bits() == old(self).spec_set(capability),
            self.spec_has(capability),
    {
        let d: u32 = capability.to_u32();

        proof {
            capability.lemma_discriminant_bounds();
        }

        assert(d <= 4u32);
        assert(d < 8);

        let old_bits: Ghost<u8> = Ghost(self.0);

        self.0 = self.0 | (1u8 << d);

        assert(self.0 == (old_bits@ | (1u8 << d)) as u8);

        // Prove the target bit is set.
        assert((self.0 & (1u8 << d)) != 0u8) by (bit_vector)
            requires
                self.0 == (old_bits@ | (1u8 << d)) as u8,
                0 <= d <= 4,
        ;
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
    /// - All other bits are unchanged.
    pub fn clear(&mut self, capability: Capability)
        ensures
            self.spec_bits() == old(self).spec_clear(capability),
            !self.spec_has(capability),
    {
        let d: u32 = capability.to_u32();

        proof {
            capability.lemma_discriminant_bounds();
        }

        assert(d <= 4u32);
        assert(d < 8);

        let old_bits: Ghost<u8> = Ghost(self.0);

        self.0 = self.0 & !(1u8 << d);

        assert(self.0 == (old_bits@ & !(1u8 << d)) as u8);

        // Prove the target bit is cleared.
        assert((self.0 & (1u8 << d)) == 0u8) by (bit_vector)
            requires
                self.0 == (old_bits@ & !(1u8 << d)) as u8,
                0 <= d <= 4,
        ;
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
        let d: u32 = capability.to_u32();

        proof {
            capability.lemma_discriminant_bounds();
        }

        assert(d <= 4u32);
        assert(d < 8);
        (self.0 & (1u8 << d)) != 0u8
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
    {
        Capabilities(0u8)
    }
}

} // verus!
