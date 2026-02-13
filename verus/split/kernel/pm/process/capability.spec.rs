// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Capabilities Specification.
// This file contains spec functions for the Capabilities type.
//
// ## Abstraction Strategy
//
// This module provides two levels of specification:
//
// 1. **Bit-level specs** (`spec_bits`, `spec_mask`, `spec_has`, `spec_set`,
//    `spec_clear`): directly model the implementation's u8 bitfield. These are
//    used in the exec-level ensures clauses and proof lemmas.
//
// 2. **Set-level specs** (`spec_as_set`, `spec_set_insert`, `spec_set_remove`,
//    `spec_set_contains`): model capabilities as a `Set<Capability>`. This is
//    the abstract interface for downstream modules. Instead of reasoning about
//    bit manipulation, callers reason about set membership, insertion, and
//    removal.
//
// The bridging lemmas (`lemma_set_insert_matches_bit_set`, etc.) prove that
// the set-level and bit-level views are consistent. This allows the exec code
// to keep its bit-level implementation while callers verify against the
// set abstraction — addressing the "spec = implementation mirror" concern.

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of Capabilities as a set of granted capabilities.
///
/// # Description
///
/// The primary abstract representation is `granted: Set<Capability>`, which
/// models capabilities as a mathematical set. The `bits` field is retained
/// for bridging proofs between the set abstraction and the bit-level
/// implementation.
#[verifier::ext_equal]
pub struct CapabilitiesView {
    /// The raw bitfield value (implementation-level).
    pub bits: u8,
    /// The set of granted capabilities (abstract-level).
    pub granted: Set<Capability>,
}

//==================================================================================================
// Spec Functions — Bit-Level (Implementation)
//==================================================================================================

impl Capabilities {
    /// Spec function: returns the raw bitfield value.
    pub open spec fn spec_bits(&self) -> u8 {
        self.bits
    }

    /// Spec function: returns the bitmask for a given capability.
    ///
    /// Maps each variant to its corresponding single-bit mask:
    /// ExceptionControl -> 0x01, InterruptControl -> 0x02, IoManagement -> 0x04,
    /// MemoryManagement -> 0x08, ProcessManagement -> 0x10.
    ///
    /// # Note
    ///
    /// Uses explicit match rather than `1 << discriminant` to avoid dependence
    /// on enum layout or `#[repr]` annotations. The equivalence to the original
    /// source's shift-based formula is proven by `lemma_mask_matches_discriminant`.
    pub open spec fn spec_mask(cap: Capability) -> u8 {
        match cap {
            Capability::ExceptionControl => 1u8,
            Capability::InterruptControl => 2u8,
            Capability::IoManagement => 4u8,
            Capability::MemoryManagement => 8u8,
            Capability::ProcessManagement => 16u8,
        }
    }

    /// Spec function: maps a discriminant value to its power-of-2 bitmask.
    ///
    /// This is the mathematical `2^d` for valid discriminants (0..=4),
    /// corresponding to `1u8 << d` in the original source. It bridges the
    /// gap between the discriminant-based formula and the explicit mask values.
    pub open spec fn spec_pow2_mask(d: int) -> u8
        recommends 0 <= d <= 4
    {
        if d == 0 { 1u8 }
        else if d == 1 { 2u8 }
        else if d == 2 { 4u8 }
        else if d == 3 { 8u8 }
        else if d == 4 { 16u8 }
        else { arbitrary() }
    }

    /// Spec function: checks whether a specific capability bit is set.
    pub open spec fn spec_has(&self, cap: Capability) -> bool {
        (self.bits & Self::spec_mask(cap)) != 0u8
    }

    /// Spec function: returns the bitfield after setting a capability bit.
    pub open spec fn spec_set(&self, cap: Capability) -> u8 {
        (self.bits | Self::spec_mask(cap)) as u8
    }

    /// Spec function: returns the bitfield after clearing a capability bit.
    pub open spec fn spec_clear(&self, cap: Capability) -> u8 {
        (self.bits & !Self::spec_mask(cap)) as u8
    }

    /// Spec function: well-formedness predicate.
    ///
    /// # Note
    ///
    /// A Capabilities value is well-formed when only valid capability bits
    /// (0..=4) are set. The upper 3 bits (5, 6, 7) must not be set.
    /// All constructors (`new`, `default`) produce well-formed values, and
    /// `set`/`clear` preserve well-formedness (proven as conditional postcondition).
    ///
    /// This is the module-level invariant. Values constructed via the public API
    /// always satisfy `wf()`. The `pub bits` field allows constructing non-`wf`
    /// values, but such values are outside the intended usage; the verification
    /// guarantees correctness for the API-reachable state space.
    pub open spec fn wf(&self) -> bool {
        self.bits & 0b1110_0000u8 == 0u8
    }

    /// Spec function: predicate asserting that every valid capability mask
    /// only uses bits in the lower 5 positions (0..=4).
    ///
    /// # Note
    ///
    /// This is a consequence of the closed-world enum: since all 5 discriminants
    /// are in [0, 4], all masks are powers of 2 up to 2^4 = 16, and none set
    /// bits 5, 6, or 7. This connects `spec_mask` to `wf()`.
    pub open spec fn spec_mask_is_valid(cap: Capability) -> bool {
        Self::spec_mask(cap) & 0b1110_0000u8 == 0u8
    }

//==================================================================================================
// Spec Functions — Set-Level (Abstract)
//==================================================================================================

    /// Spec function: converts the capabilities bitfield to a set of Capability values.
    ///
    /// # Description
    ///
    /// This is the primary abstract representation. Instead of reasoning about
    /// bit positions, downstream modules can reason about set membership:
    ///   `self.spec_as_set().contains(cap)  <==>  self.spec_has(cap)`
    ///
    /// This addresses the "spec = implementation mirror" concern by providing
    /// a higher-level abstraction: capabilities are a *set*, not a bitfield.
    pub open spec fn spec_as_set(&self) -> Set<Capability> {
        Set::empty()
            .insert_if(self.spec_has(Capability::ExceptionControl), Capability::ExceptionControl)
            .insert_if(self.spec_has(Capability::InterruptControl), Capability::InterruptControl)
            .insert_if(self.spec_has(Capability::IoManagement), Capability::IoManagement)
            .insert_if(self.spec_has(Capability::MemoryManagement), Capability::MemoryManagement)
            .insert_if(self.spec_has(Capability::ProcessManagement), Capability::ProcessManagement)
    }

    /// Spec function: checks whether a capability is in the granted set.
    ///
    /// # Description
    ///
    /// Abstract predicate equivalent to `spec_has`, expressed in set terms.
    /// Downstream modules should prefer this over `spec_has` for cleaner specs.
    pub open spec fn spec_set_contains(&self, cap: Capability) -> bool {
        self.spec_as_set().contains(cap)
    }

    /// Spec function: returns the set after granting a capability.
    ///
    /// # Description
    ///
    /// Models `set(cap)` as set insertion. Downstream modules can write:
    ///   `ensures self.spec_granted() == old(self).spec_granted().insert(cap)`
    /// instead of reasoning about bit-level OR operations.
    pub open spec fn spec_set_insert(&self, cap: Capability) -> Set<Capability> {
        self.spec_as_set().insert(cap)
    }

    /// Spec function: returns the set after revoking a capability.
    ///
    /// # Description
    ///
    /// Models `clear(cap)` as set removal. Downstream modules can write:
    ///   `ensures self.spec_granted() == old(self).spec_granted().remove(cap)`
    /// instead of reasoning about bit-level AND-NOT operations.
    pub open spec fn spec_set_remove(&self, cap: Capability) -> Set<Capability> {
        self.spec_as_set().remove(cap)
    }

    /// Spec function: the set of granted capabilities (alias for `spec_as_set`).
    ///
    /// # Description
    ///
    /// Convenience alias. The canonical abstract postcondition for downstream
    /// modules is:
    ///   `process.capabilities().spec_granted().contains(Capability::ProcessManagement)`
    /// rather than:
    ///   `(process.capabilities().bits & 16u8) != 0u8`
    pub open spec fn spec_granted(&self) -> Set<Capability> {
        self.spec_as_set()
    }

    /// Spec function: the default (empty) capabilities as a set.
    pub open spec fn spec_empty_set() -> Set<Capability> {
        Set::empty()
    }

    /// Spec function: the number of granted capabilities.
    pub open spec fn spec_count(&self) -> nat {
        (if self.spec_has(Capability::ExceptionControl) { 1nat } else { 0nat })
        + (if self.spec_has(Capability::InterruptControl) { 1nat } else { 0nat })
        + (if self.spec_has(Capability::IoManagement) { 1nat } else { 0nat })
        + (if self.spec_has(Capability::MemoryManagement) { 1nat } else { 0nat })
        + (if self.spec_has(Capability::ProcessManagement) { 1nat } else { 0nat })
    }
}

//==================================================================================================
// Helper Spec for Set Construction
//==================================================================================================

/// Extension trait for conditional set insertion (used by spec_as_set).
pub trait SetInsertIf<T> {
    /// Inserts the element if the condition is true, otherwise returns self unchanged.
    spec fn insert_if(self, cond: bool, elem: T) -> Self;
}

impl<T> SetInsertIf<T> for Set<T> {
    open spec fn insert_if(self, cond: bool, elem: T) -> Set<T> {
        if cond { self.insert(elem) } else { self }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Capabilities {
    type V = CapabilitiesView;

    open spec fn view(&self) -> CapabilitiesView {
        CapabilitiesView {
            bits: self.bits,
            granted: self.spec_as_set(),
        }
    }
}

} // verus!
