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
    /// Invariant predicate for internal consistency.
    ///
    /// # Description
    ///
    /// Enforces that the signal count never exceeds the total. This captures the
    /// reachable-state invariant:
    /// - `new(total)` produces `count == 0`, so `count <= total` holds.
    /// - `signal()` increments `count` by 1, but only when `count < total`.
    /// - `wait()` is a pure observer and does not modify state.
    pub closed spec fn inv(&self) -> bool {
        self.count as nat <= self.total as nat
    }

    /// Spec function: returns whether the fence is satisfied (all signals received).
    pub open spec fn spec_is_satisfied(&self) -> bool {
        self.count as nat >= self.total as nat
    }

    /// Spec function: returns whether the fence is waiting (not yet satisfied).
    pub open spec fn spec_is_waiting(&self) -> bool {
        (self.count as nat) < (self.total as nat)
    }

    /// Spec function: returns the number of signals received.
    pub open spec fn spec_count(&self) -> nat {
        self.count as nat
    }

    /// Spec function: returns the total number of signals required.
    pub open spec fn spec_total(&self) -> nat {
        self.total as nat
    }

    /// Spec function: returns the number of remaining signals needed.
    ///
    /// # Note
    ///
    /// This function assumes `inv()` for meaningful results. Without `inv()`,
    /// if `count > total`, nat subtraction saturates to 0.
    pub open spec fn spec_remaining(&self) -> nat
        recommends self.inv(),
    {
        (self.total as nat - self.count as nat) as nat
    }

    /// Spec function: the view of a newly created fence with the given total.
    pub open spec fn spec_new_view(total: nat) -> FenceView {
        FenceView { count: 0, total: total }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

/// NOTE: `view()` must be `open spec fn` because the Verus `View` trait requires it.
/// The trait signature mandates `open`, so this cannot be `closed`. Users observe
/// only the abstract `FenceView` (which uses `nat` instead of `usize`), not the
/// concrete `Fence` fields directly.
impl View for Fence {
    type V = FenceView;

    open spec fn view(&self) -> FenceView {
        FenceView { count: self.count as nat, total: self.total as nat }
    }
}

} // verus!
