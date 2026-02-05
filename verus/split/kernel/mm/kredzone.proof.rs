// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {


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

} // verus!
