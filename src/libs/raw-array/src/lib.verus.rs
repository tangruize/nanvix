// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RawArray Verification Specifications
//
// This file contains complete Verus verification specifications for RawArray.
// All spec functions, proof lemmas, and tests are included.

use vstd::prelude::*;

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

/// Predicate: All elements in the array are zero-initialized.
pub open spec fn all_zeros<T>(view: &RawArrayView<T>) -> bool {
    forall|i: int| 0 <= i < view.len() as int ==> is_zero(#[trigger] view.index(i))
}

//==================================================================================================
// RawArray Invariant
//==================================================================================================

/// The invariant that all RawArray instances must satisfy.
/// This is trivially true for our abstraction, but captures key properties.
pub open spec fn raw_array_inv<T>(len: nat) -> bool {
    // Length must be positive and bounded.
    &&& len > 0
    &&& len < i32::MAX as nat
}

//==================================================================================================
// Additional Lemmas for Client Code
//==================================================================================================

/// Lemma: If two arrays have the same view, they are equivalent.
pub proof fn lemma_view_determines_equivalence<T>(v1: Seq<T>, v2: Seq<T>)
    requires
        v1 =~= v2,
    ensures
        v1.len() == v2.len(),
        forall|i: int| 0 <= i < v1.len() as int ==> v1[i] == v2[i],
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
// Tests
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
        let updated: RawArrayView<u8> = view.update(1, 5u8);
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
        let view2: RawArrayView<u8> = view;
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

    /// Test: Multiple updates preserve length.
    proof fn test_multiple_updates_preserve_len() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![0u8, 0u8, 0u8, 0u8, 0u8],
        };

        let v1: RawArrayView<u8> = view.update(0, 1u8);
        let v2: RawArrayView<u8> = v1.update(2, 2u8);
        let v3: RawArrayView<u8> = v2.update(4, 3u8);

        lemma_update_preserves_len(view, 0, 1u8);
        lemma_update_preserves_len(v1, 2, 2u8);
        lemma_update_preserves_len(v2, 4, 3u8);

        assert(v3.len() == view.len());
        assert(v3.len() == 5);
    }

    /// Test: Update at different indices results in independent changes.
    proof fn test_independent_updates() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![0u8, 0u8, 0u8, 0u8],
        };

        let v1: RawArrayView<u8> = view.update(0, 10u8);
        let v2: RawArrayView<u8> = v1.update(3, 20u8);

        // Both values are set correctly.
        lemma_update_sets_index(view, 0, 10u8);
        assert(v1.index(0) == 10u8);

        lemma_update_only_changes_index(v1, 3, 20u8, 0);
        assert(v2.index(0) == 10u8);

        lemma_update_sets_index(v1, 3, 20u8);
        assert(v2.index(3) == 20u8);
    }

    /// Test: View transitivity.
    proof fn test_view_eq_transitivity() {
        let v1: RawArrayView<u8> = RawArrayView { contents: seq![1u8, 2u8] };
        let v2: RawArrayView<u8> = RawArrayView { contents: seq![1u8, 2u8] };
        let v3: RawArrayView<u8> = RawArrayView { contents: seq![1u8, 2u8] };

        // v1 == v2.
        assert(v1.spec_eq(&v2));
        // v2 == v3.
        assert(v2.spec_eq(&v3));
        // Therefore v1 == v3.
        lemma_view_eq_transitive(&v1, &v2, &v3);
        assert(v1.spec_eq(&v3));
    }

    /// Test: Equal views have equal elements at all indices.
    proof fn test_equal_views_all_elements() {
        let v1: RawArrayView<u8> = RawArrayView { contents: seq![5u8, 10u8, 15u8] };
        let v2: RawArrayView<u8> = RawArrayView { contents: seq![5u8, 10u8, 15u8] };

        assert(v1.spec_eq(&v2));

        lemma_equal_views_equal_elements(&v1, &v2, 0);
        lemma_equal_views_equal_elements(&v1, &v2, 1);
        lemma_equal_views_equal_elements(&v1, &v2, 2);

        assert(v1.index(0) == v2.index(0));
        assert(v1.index(1) == v2.index(1));
        assert(v1.index(2) == v2.index(2));
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

    /// Test: Update does not affect unrelated indices (frame condition).
    proof fn test_update_frame_condition() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![1u8, 2u8, 3u8, 4u8, 5u8],
        };

        let updated: RawArrayView<u8> = view.update(2, 99u8);

        // Indices 0, 1, 3, 4 are unchanged.
        lemma_update_only_changes_index(view, 2, 99u8, 0);
        lemma_update_only_changes_index(view, 2, 99u8, 1);
        lemma_update_only_changes_index(view, 2, 99u8, 3);
        lemma_update_only_changes_index(view, 2, 99u8, 4);

        assert(updated.index(0) == 1u8);
        assert(updated.index(1) == 2u8);
        assert(updated.index(3) == 4u8);
        assert(updated.index(4) == 5u8);

        // Index 2 is changed.
        lemma_update_sets_index(view, 2, 99u8);
        assert(updated.index(2) == 99u8);
    }
}

} // verus!
