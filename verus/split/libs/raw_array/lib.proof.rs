// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RawArray - Proofs
//
// This file contains lemmas and proof functions for RawArray.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Lemmas about RawArrayView
//==================================================================================================

/// Lemma: Updating an array preserves its length.
pub proof fn lemma_update_preserves_len<T>(view: RawArrayView<T>, i: int, value: T)
    requires
        0 <= i < view.len() as int,
    ensures
        view.update(i, value).len() == view.len(),
{
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
}

/// Lemma: Updating index i sets index i to the new value.
pub proof fn lemma_update_sets_index<T>(view: RawArrayView<T>, i: int, value: T)
    requires
        0 <= i < view.len() as int,
    ensures
        view.update(i, value).index(i) == value,
{
}

//==================================================================================================
// Lemmas about View Equality
//==================================================================================================

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
pub proof fn lemma_view_eq_transitive<T>(
    v1: &RawArrayView<T>,
    v2: &RawArrayView<T>,
    v3: &RawArrayView<T>,
)
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
// Lemmas for Seq Operations
//==================================================================================================

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

/// Lemma: Setting an element preserves all other elements (frame condition).
pub proof fn lemma_set_frame<T>(view: Seq<T>, i: int, value: T)
    requires
        0 <= i < view.len(),
    ensures
        view.update(i, value).len() == view.len(),
        forall|j: int| 0 <= j < view.len() && j != i
            ==> #[trigger] view.update(i, value)[j] == view[j],
{
}

//==================================================================================================
// Tests
//==================================================================================================

#[cfg(verus_keep_ghost)]
mod test {
    use super::*;

    proof fn test_view_update_properties() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![0u8, 0u8, 0u8, 0u8],
        };

        assert(view.len() == 4);

        let updated: RawArrayView<u8> = view.update(1, 5u8);
        lemma_update_preserves_len(view, 1, 5u8);
        assert(updated.len() == 4);

        lemma_update_sets_index(view, 1, 5u8);
        assert(updated.index(1) == 5u8);

        lemma_update_only_changes_index(view, 1, 5u8, 0);
        assert(updated.index(0) == view.index(0));
    }

    proof fn test_equality_lemmas() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![1u8, 2u8, 3u8],
        };

        lemma_view_eq_reflexive(&view);
        assert(view.spec_eq(&view));

        let view2: RawArrayView<u8> = view;
        lemma_equal_views_equal_len(&view, &view2);
        assert(view.len() == view2.len());
    }

    proof fn test_set_commutes() {
        let view: Seq<u8> = seq![0u8, 0u8, 0u8, 0u8];

        lemma_set_commutes(view, 0, 1u8, 2, 3u8);
        assert(view.update(0, 1u8).update(2, 3u8) =~= view.update(2, 3u8).update(0, 1u8));
    }

    proof fn test_set_overwrite() {
        let view: Seq<u8> = seq![0u8, 0u8, 0u8, 0u8];

        lemma_set_overwrite(view, 1, 5u8, 10u8);
        assert(view.update(1, 5u8).update(1, 10u8) =~= view.update(1, 10u8));
    }

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

    proof fn test_independent_updates() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![0u8, 0u8, 0u8, 0u8],
        };

        let v1: RawArrayView<u8> = view.update(0, 10u8);
        let v2: RawArrayView<u8> = v1.update(3, 20u8);

        lemma_update_sets_index(view, 0, 10u8);
        assert(v1.index(0) == 10u8);

        lemma_update_only_changes_index(v1, 3, 20u8, 0);
        assert(v2.index(0) == 10u8);

        lemma_update_sets_index(v1, 3, 20u8);
        assert(v2.index(3) == 20u8);
    }

    proof fn test_view_eq_transitivity() {
        let v1: RawArrayView<u8> = RawArrayView { contents: seq![1u8, 2u8] };
        let v2: RawArrayView<u8> = RawArrayView { contents: seq![1u8, 2u8] };
        let v3: RawArrayView<u8> = RawArrayView { contents: seq![1u8, 2u8] };

        assert(v1.spec_eq(&v2));
        assert(v2.spec_eq(&v3));
        lemma_view_eq_transitive(&v1, &v2, &v3);
        assert(v1.spec_eq(&v3));
    }

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

    proof fn test_raw_array_inv() {
        assert(raw_array_inv::<u8>(1));
        assert(raw_array_inv::<u8>(100));
        assert(raw_array_inv::<u8>(1000000));
        assert(!raw_array_inv::<u8>(0));
    }

    proof fn test_update_frame_condition() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![1u8, 2u8, 3u8, 4u8, 5u8],
        };

        let updated: RawArrayView<u8> = view.update(2, 99u8);

        lemma_update_only_changes_index(view, 2, 99u8, 0);
        lemma_update_only_changes_index(view, 2, 99u8, 1);
        lemma_update_only_changes_index(view, 2, 99u8, 3);
        lemma_update_only_changes_index(view, 2, 99u8, 4);

        assert(updated.index(0) == 1u8);
        assert(updated.index(1) == 2u8);
        assert(updated.index(3) == 4u8);
        assert(updated.index(4) == 5u8);

        lemma_update_sets_index(view, 2, 99u8);
        assert(updated.index(2) == 99u8);
    }
}

} // verus!
