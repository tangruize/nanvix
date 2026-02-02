// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # RawArray Verification Specifications
//!
//! This file contains Verus verification specifications, proofs, and lemmas
//! for the RawArray module. It is included via `include!` macro only when
//! `verus_keep_ghost` is defined.
//!
//! ## Memory Safety Model
//!
//! RawArray provides a safe abstraction over raw memory with the following guarantees:
//! 1. **Length invariance**: Array length never changes after construction
//! 2. **Element isolation**: Writing to one index doesn't affect others
//! 3. **Bounds safety**: All accesses are within bounds
//! 4. **Initial state**: New arrays are zero-initialized

use ::sys::error::ErrorCode;

verus! {

//==================================================================================================
// RawArrayView - The Abstract Specification Model
//==================================================================================================

/// Abstract view of a RawArray as a sequence.
///
/// This is the ghost/spec-level representation used for verification.
/// All properties are stated in terms of this view.
pub ghost struct RawArrayView<T> {
    /// The logical contents of the array.
    pub contents: Seq<T>,
}

impl<T> RawArrayView<T> {
    /// Returns the length of the array view.
    pub open spec fn len(&self) -> nat {
        self.contents.len()
    }

    /// Returns the element at index i.
    pub open spec fn index(&self, i: int) -> T
        recommends 0 <= i < self.len() as int
    {
        self.contents[i]
    }

    /// Returns a new view with the element at index i updated to value.
    pub open spec fn update(&self, i: int, value: T) -> RawArrayView<T>
        recommends 0 <= i < self.len() as int
    {
        RawArrayView {
            contents: self.contents.update(i, value),
        }
    }

    /// Two views are equal if they have the same contents.
    pub open spec fn spec_eq(&self, other: &RawArrayView<T>) -> bool {
        self.contents =~= other.contents
    }
}

//==================================================================================================
// Lemmas about RawArrayView - Fully Verified
//==================================================================================================

/// Lemma: Updating an array preserves its length.
pub proof fn lemma_update_preserves_len<T>(view: RawArrayView<T>, i: int, value: T)
    requires
        0 <= i < view.len() as int,
    ensures
        view.update(i, value).len() == view.len(),
{
    // Follows from Seq::update preserving length.
}

/// Lemma: Updating index i only changes index i.
pub proof fn lemma_update_only_changes_index<T>(view: RawArrayView<T>, i: int, value: T, j: int)
    requires
        0 <= i < view.len() as int,
        0 <= j < view.len() as int,
        i != j,
    ensures
        view.update(i, value).index(j) == view.index(j),
{
    // Follows from Seq::update property.
}

/// Lemma: Updating index i sets index i to the new value.
pub proof fn lemma_update_sets_index<T>(view: RawArrayView<T>, i: int, value: T)
    requires
        0 <= i < view.len() as int,
    ensures
        view.update(i, value).index(i) == value,
{
    // Follows from Seq::update property.
}

/// Lemma: View equality is reflexive.
pub proof fn lemma_view_eq_reflexive<T>(view: &RawArrayView<T>)
    ensures
        view.spec_eq(view),
{
}

/// Lemma: View equality is symmetric.
pub proof fn lemma_view_eq_symmetric<T>(v1: &RawArrayView<T>, v2: &RawArrayView<T>)
    requires
        v1.spec_eq(v2),
    ensures
        v2.spec_eq(v1),
{
}

/// Lemma: View equality is transitive.
pub proof fn lemma_view_eq_transitive<T>(v1: &RawArrayView<T>, v2: &RawArrayView<T>, v3: &RawArrayView<T>)
    requires
        v1.spec_eq(v2),
        v2.spec_eq(v3),
    ensures
        v1.spec_eq(v3),
{
}

/// Lemma: Equal views have equal elements.
pub proof fn lemma_equal_views_equal_elements<T>(v1: &RawArrayView<T>, v2: &RawArrayView<T>, i: int)
    requires
        v1.spec_eq(v2),
        0 <= i < v1.len() as int,
    ensures
        v1.index(i) == v2.index(i),
{
}

/// Lemma: Equal views have equal lengths.
pub proof fn lemma_equal_views_equal_len<T>(v1: &RawArrayView<T>, v2: &RawArrayView<T>)
    requires
        v1.spec_eq(v2),
    ensures
        v1.len() == v2.len(),
{
}

//==================================================================================================
// Zero Initialization Specification
//==================================================================================================

/// Predicate: Specifies that a value is the zero/default value for its type.
/// This is used to specify that newly allocated arrays are zero-initialized.
pub uninterp spec fn is_zero<T>(value: T) -> bool;

/// Axiom: For u8, zero means the value is 0.
pub axiom fn axiom_u8_zero_is_0(t: u8)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For u16, zero means the value is 0.
pub axiom fn axiom_u16_zero_is_0(t: u16)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For u32, zero means the value is 0.
pub axiom fn axiom_u32_zero_is_0(t: u32)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For u64, zero means the value is 0.
pub axiom fn axiom_u64_zero_is_0(t: u64)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For usize, zero means the value is 0.
pub axiom fn axiom_usize_zero_is_0(t: usize)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For i8, zero means the value is 0.
pub axiom fn axiom_i8_zero_is_0(t: i8)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For i16, zero means the value is 0.
pub axiom fn axiom_i16_zero_is_0(t: i16)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For i32, zero means the value is 0.
pub axiom fn axiom_i32_zero_is_0(t: i32)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For i64, zero means the value is 0.
pub axiom fn axiom_i64_zero_is_0(t: i64)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For isize, zero means the value is 0.
pub axiom fn axiom_isize_zero_is_0(t: isize)
    requires is_zero(t)
    ensures t == 0;

/// Predicate: All elements in the array are zero-initialized.
pub open spec fn all_zeros<T>(view: &RawArrayView<T>) -> bool {
    forall|i: int| 0 <= i < view.len() as int ==> is_zero(#[trigger] view.index(i))
}

//==================================================================================================
// RawArray Invariant - Verified
//==================================================================================================

/// The invariant that all RawArray instances must satisfy.
/// This is trivially true for our abstraction, but captures key properties.
pub open spec fn raw_array_inv<T>(len: nat) -> bool {
    // Length must be positive and bounded.
    &&& len > 0
    &&& len < i32::MAX as nat
}

//==================================================================================================
// RawArray Specification Functions
//==================================================================================================

impl<T> RawArray<T> {
    /// Returns the spec-level length of the array.
    pub open spec fn spec_len(&self) -> nat {
        self@.len()
    }

    /// Returns true if index i is within bounds.
    pub open spec fn in_bounds(&self, i: int) -> bool {
        0 <= i < self.spec_len() as int
    }

    /// Invariant for RawArray: length is positive and bounded.
    pub open spec fn inv(&self) -> bool {
        &&& self@.len() > 0
        &&& self@.len() < i32::MAX as nat
    }
}

//==================================================================================================
// View Implementation for RawArray
//==================================================================================================

impl<T> View for RawArray<T> {
    type V = Seq<T>;

    /// The abstract view of the array as a sequence.
    ///
    /// This is uninterpreted because the actual mapping from raw memory to Seq
    /// requires trusting the memory subsystem.
    uninterp spec fn view(&self) -> Seq<T>;
}

//==================================================================================================
// Specification for new() - Constructor
//==================================================================================================

/// Specification for RawArray::new().
pub open spec fn spec_new_ensures<T>(len: usize, result: Result<RawArray<T>, Error>) -> bool {
    // Success case: array is properly initialized.
    &&& (result is Ok ==> {
        &&& result->Ok_0.inv()
        &&& result->Ok_0@.len() == len
        &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
    })
    // Error case: only OutOfMemory is possible when preconditions are met.
    &&& (result is Err ==> result->Err_0.code == ErrorCode::OutOfMemory)
}

/// Specification for RawArray::from_raw_parts().
pub open spec fn spec_from_raw_parts_ensures<T>(len: usize, result: Result<RawArray<T>, Error>) -> bool {
    // Success case: array is properly initialized.
    &&& (result is Ok ==> {
        &&& result->Ok_0.inv()
        &&& result->Ok_0@.len() == len
        &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
    })
    // Error case: only InvalidArgument is possible (null or wrapping pointer).
    &&& (result is Err ==> result->Err_0.code == ErrorCode::InvalidArgument)
}

//==================================================================================================
// Additional Lemmas for Client Code
//==================================================================================================

/// Lemma: If two arrays have the same view, they are equivalent.
pub proof fn lemma_view_determines_equivalence<T>(a1: &RawArray<T>, a2: &RawArray<T>)
    requires
        a1@ =~= a2@,
    ensures
        a1.spec_len() == a2.spec_len(),
        forall|i: int| 0 <= i < a1.spec_len() as int ==> a1@[i] == a2@[i],
{
}

/// Lemma: Consecutive sets to different indices commute.
pub proof fn lemma_set_commutes<T>(view: Seq<T>, i: int, vi: T, j: int, vj: T)
    requires
        0 <= i < view.len(),
        0 <= j < view.len(),
        i != j,
    ensures
        view.update(i, vi).update(j, vj) =~= view.update(j, vj).update(i, vi),
{
}

/// Lemma: Setting the same index twice results in the last value.
pub proof fn lemma_set_overwrite<T>(view: Seq<T>, i: int, v1: T, v2: T)
    requires
        0 <= i < view.len(),
    ensures
        view.update(i, v1).update(i, v2) =~= view.update(i, v2),
{
}

//==================================================================================================
// Tests (Verification Only)
//==================================================================================================

#[cfg(verus_keep_ghost)]
mod test {
    use super::*;

    /// Test: RawArrayView update properties.
    proof fn test_view_update_properties() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![0u8, 0u8, 0u8, 0u8],
        };

        // Length is 4.
        assert(view.len() == 4);

        // Update preserves length.
        let updated = view.update(1, 5u8);
        lemma_update_preserves_len(view, 1, 5u8);
        assert(updated.len() == 4);

        // Update sets the target index.
        lemma_update_sets_index(view, 1, 5u8);
        assert(updated.index(1) == 5u8);

        // Update preserves other indices.
        lemma_update_only_changes_index(view, 1, 5u8, 0);
        assert(updated.index(0) == view.index(0));
    }

    /// Test: Equality lemmas.
    proof fn test_equality_lemmas() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![1u8, 2u8, 3u8],
        };

        // Reflexive.
        lemma_view_eq_reflexive(&view);
        assert(view.spec_eq(&view));

        // Equal views have equal lengths.
        let view2 = view;
        lemma_equal_views_equal_len(&view, &view2);
        assert(view.len() == view2.len());
    }

    /// Test: Set commutes lemma.
    proof fn test_set_commutes() {
        let view: Seq<u8> = seq![0u8, 0u8, 0u8, 0u8];

        lemma_set_commutes(view, 0, 1u8, 2, 3u8);
        assert(view.update(0, 1u8).update(2, 3u8) =~= view.update(2, 3u8).update(0, 1u8));
    }

    /// Test: Set overwrite lemma.
    proof fn test_set_overwrite() {
        let view: Seq<u8> = seq![0u8, 0u8, 0u8, 0u8];

        lemma_set_overwrite(view, 1, 5u8, 10u8);
        assert(view.update(1, 5u8).update(1, 10u8) =~= view.update(1, 10u8));
    }

    /// Test: Invariant properties.
    proof fn test_raw_array_inv() {
        // Valid invariant.
        assert(raw_array_inv::<u8>(1));
        assert(raw_array_inv::<u8>(100));
        assert(raw_array_inv::<u8>(1000000));

        // Invalid invariant (length 0).
        assert(!raw_array_inv::<u8>(0));
    }
}

} // verus!
