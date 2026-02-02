// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RawArray Verification Specifications

use vstd::prelude::*;

verus! {

//==================================================================================================
// RawArrayView - The Abstract Specification Model
//==================================================================================================

/// Abstract view of a RawArray as a sequence.
pub ghost struct RawArrayView<T> {
    pub contents: Seq<T>,
}

impl<T> RawArrayView<T> {
    /// Returns the length of the array view.
    pub open spec fn len(&self) -> nat {
        self.contents.len()
    }

    /// Returns the element at index i.
    pub open spec fn index(&self, i: int) -> T {
        self.contents[i]
    }

    /// Returns a new view with the element at index i updated to value.
    pub open spec fn update(&self, i: int, value: T) -> RawArrayView<T> {
        RawArrayView {
            contents: self.contents.update(i, value),
        }
    }
}

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
// Zero Initialization Specification
//==================================================================================================

/// Predicate: Specifies that a value is the zero/default value for its type.
pub uninterp spec fn is_zero<T>(value: T) -> bool;

/// Predicate: All elements in the array are zero-initialized.
pub open spec fn all_zeros<T>(view: &RawArrayView<T>) -> bool {
    forall|i: int| 0 <= i < view.len() as int ==> is_zero(#[trigger] view.index(i))
}

//==================================================================================================
// RawArray Invariant
//==================================================================================================

/// The invariant that all RawArray instances must satisfy.
pub open spec fn raw_array_inv<T>(len: nat) -> bool {
    &&& len > 0
    &&& len < i32::MAX as nat
}

//==================================================================================================
// Additional Lemmas
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

} // verus!
