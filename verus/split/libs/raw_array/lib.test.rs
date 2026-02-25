// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RawArray - Verified Tests.
// Verified test functions that prove key RawArray properties.

verus! {

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
