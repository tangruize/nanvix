// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Fence Specification.
// This file contains spec functions for the Fence type.

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a Fence.
///
/// # Description
///
/// Represents the observable state of a fence: how many signals have been received
/// (`count`) and how many are required (`total`).
#[verifier::ext_equal]
pub struct FenceView {
    /// Number of signals received so far.
    pub count: nat,
    /// Total number of signals required before the fence is satisfied.
    pub total: nat,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl Fence {
    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// Enforces that the signal count never exceeds the total. This captures the
    /// reachable-state invariant:
    /// - `new(total)` produces `count == 0`, so `count <= total` holds.
    /// - `signal()` increments `count` by 1, but only when `count < total`.
    /// - `wait()` is a pure observer and does not modify state.
    pub open spec fn wf(&self) -> bool {
        self.count@ <= self.total@
    }

    /// Spec function: returns whether the fence is satisfied (all signals received).
    pub open spec fn spec_is_satisfied(&self) -> bool {
        self.count@ >= self.total@
    }

    /// Spec function: returns whether the fence is waiting (not yet satisfied).
    pub open spec fn spec_is_waiting(&self) -> bool {
        self.count@ < self.total@
    }

    /// Spec function: returns the number of signals received.
    pub open spec fn spec_count(&self) -> nat {
        self.count@
    }

    /// Spec function: returns the total number of signals required.
    pub open spec fn spec_total(&self) -> nat {
        self.total@
    }

    /// Spec function: returns the number of remaining signals needed.
    pub open spec fn spec_remaining(&self) -> nat {
        (self.total@ - self.count@) as nat
    }

    /// Spec function: the view of a newly created fence with the given total.
    pub open spec fn spec_new_view(total: nat) -> FenceView {
        FenceView { count: 0, total: total }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Fence {
    type V = FenceView;

    open spec fn view(&self) -> FenceView {
        FenceView { count: self.count@, total: self.total@ }
    }
}

} // verus!
